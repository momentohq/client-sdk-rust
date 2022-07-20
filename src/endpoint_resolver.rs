use crate::jwt::{decode_jwt, Claims};
use crate::response::error::MomentoError;

#[derive(Debug)]
pub struct MomentoEndpoint {
    pub url: String,
    pub hostname: String,
}

#[derive(Debug)]
pub struct MomentoEndpoints {
    pub control_endpoint: MomentoEndpoint,
    pub data_endpoint: MomentoEndpoint,
}

pub struct MomentoEndpointsResolver {}

const CONTROL_ENDPOINT_PREFIX: &str = "control.";
const DATA_ENDPOINT_PREFIX: &str = "data.";
const LOGIN_HOSTNAMES: &[&str] = &["control.cell-us-east-1-1.prod.a.momentohq.com"];

impl MomentoEndpointsResolver {
    pub fn resolve(
        auth_token: &str,
        momento_endpoint: Option<String>,
    ) -> Result<MomentoEndpoints, MomentoError> {
        let claims = match decode_jwt(auth_token, momento_endpoint.to_owned()) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        let hosted_zone = MomentoEndpointsResolver::get_hosted_zone(momento_endpoint);

        let control_endpoint =
            MomentoEndpointsResolver::get_control_endpoint(&claims, hosted_zone.to_owned());
        let data_endpoint = MomentoEndpointsResolver::get_data_endpoint(&claims, hosted_zone);

        Ok(MomentoEndpoints {
            control_endpoint,
            data_endpoint,
        })
    }

    pub fn get_login_endpoint() -> String {
        match std::env::var("LOGIN_ENDPOINT") {
            Ok(override_hostname) => override_hostname,
            Err(_) => {
                let random = rand::random::<usize>();
                let hostname = LOGIN_HOSTNAMES[random % LOGIN_HOSTNAMES.len()].to_string();
                format!("https://{hostname}:443")
            }
        }
    }

    fn get_hosted_zone(momento_endpoint: Option<String>) -> Option<String> {
        // TODO: If not a full url, lookup in the endpoint maps.
        // For now, assuming that momento_endpoint is same as the hosted zone.
        momento_endpoint
    }

    fn wrapped_endpoint(prefix: &str, hostname: String, suffix: &str) -> MomentoEndpoint {
        MomentoEndpoint {
            url: format!("{prefix}{hostname}{suffix}"),
            hostname,
        }
    }

    fn https_endpoint(hostname: String) -> MomentoEndpoint {
        MomentoEndpointsResolver::wrapped_endpoint("https://", hostname, ":443")
    }

    fn get_control_endpoint(claims: &Claims, hosted_zone: Option<String>) -> MomentoEndpoint {
        MomentoEndpointsResolver::get_control_endpoint_from_hosted_zone(hosted_zone).unwrap_or_else(
            || MomentoEndpointsResolver::https_endpoint(claims.cp.as_ref().unwrap().to_owned()),
        )
    }

    fn get_data_endpoint(claims: &Claims, hosted_zone: Option<String>) -> MomentoEndpoint {
        MomentoEndpointsResolver::get_data_endpoint_from_hosted_zone(hosted_zone).unwrap_or_else(
            || MomentoEndpointsResolver::https_endpoint(claims.c.as_ref().unwrap().to_owned()),
        )
    }

    fn hosted_zone_endpoint(hosted_zone: Option<String>, prefix: &str) -> Option<MomentoEndpoint> {
        hosted_zone.map(|hosted_zone| {
            let hostname = format!("{}{}", prefix, hosted_zone);
            MomentoEndpointsResolver::wrapped_endpoint("https://", hostname, ":443")
        })
    }

    fn get_control_endpoint_from_hosted_zone(
        hosted_zone: Option<String>,
    ) -> Option<MomentoEndpoint> {
        MomentoEndpointsResolver::hosted_zone_endpoint(hosted_zone, CONTROL_ENDPOINT_PREFIX)
    }

    fn get_data_endpoint_from_hosted_zone(hosted_zone: Option<String>) -> Option<MomentoEndpoint> {
        MomentoEndpointsResolver::hosted_zone_endpoint(hosted_zone, DATA_ENDPOINT_PREFIX)
    }
}

#[cfg(test)]
mod tests {
    use crate::endpoint_resolver::MomentoEndpointsResolver;
    use crate::response::error::MomentoError;

    #[test]
    fn urls_from_auth_token() {
        let valid_auth_token = "eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiY29udHJvbCBwbGFuZSBlbmRwb2ludCIsImMiOiJkYXRhIHBsYW5lIGVuZHBvaW50In0.zsTsEXFawetTCZI";
        let endpoints = MomentoEndpointsResolver::resolve(valid_auth_token, None);
        assert_eq!(
            endpoints.as_ref().unwrap().data_endpoint.hostname,
            "data plane endpoint"
        );
        assert_eq!(
            endpoints.as_ref().unwrap().data_endpoint.url,
            "https://data plane endpoint:443"
        );
        assert_eq!(
            endpoints.as_ref().unwrap().control_endpoint.hostname,
            "control plane endpoint"
        );
        assert_eq!(
            endpoints.as_ref().unwrap().control_endpoint.url,
            "https://control plane endpoint:443"
        );
    }

    #[test]
    fn urls_from_hosted_zone() {
        let valid_auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba";
        let endpoints = MomentoEndpointsResolver::resolve(
            valid_auth_token,
            Some("hello.gomomento.com".to_string()),
        );
        assert_eq!(
            endpoints.as_ref().unwrap().data_endpoint.hostname,
            "data.hello.gomomento.com"
        );
        assert_eq!(
            endpoints.as_ref().unwrap().data_endpoint.url,
            "https://data.hello.gomomento.com:443"
        );
        assert_eq!(
            endpoints.as_ref().unwrap().control_endpoint.hostname,
            "control.hello.gomomento.com"
        );
        assert_eq!(
            endpoints.as_ref().unwrap().control_endpoint.url,
            "https://control.hello.gomomento.com:443"
        );
    }

    #[test]
    fn error_when_no_cp_c_claims_and_no_hosted_zone() {
        let invalid_auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba";
        let e = MomentoEndpointsResolver::resolve(invalid_auth_token, None).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }
}
