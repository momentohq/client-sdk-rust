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

impl MomentoEndpointsResolver {
    pub fn resolve(
        auth_token: &str,
        hosted_zone: &Option<String>,
    ) -> Result<MomentoEndpoints, MomentoError> {
        let claims = match decode_jwt(auth_token) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        let control_endpoint = MomentoEndpointsResolver::get_control_endpoint(&claims, hosted_zone);
        let data_endpoint = MomentoEndpointsResolver::get_data_endpoint(&claims, hosted_zone);
        Ok(MomentoEndpoints {
            control_endpoint,
            data_endpoint,
        })
    }

    fn wrapped_endpoint(prefix: &str, hostname: String, suffix: &str) -> MomentoEndpoint {
        MomentoEndpoint {
            url: format!("{prefix}{hostname}{suffix}"),
            hostname: hostname,
        }
    }

    fn https_endpoint(hostname: String) -> MomentoEndpoint {
        MomentoEndpointsResolver::wrapped_endpoint("https://", hostname, ":443")
    }

    fn get_control_endpoint(claims: &Claims, hosted_zone: &Option<String>) -> MomentoEndpoint {
        MomentoEndpointsResolver::get_control_endpoint_from_hosted_zone(hosted_zone)
            .unwrap_or_else(|| MomentoEndpointsResolver::https_endpoint(claims.cp.to_owned()))
    }

    fn get_data_endpoint(claims: &Claims, hosted_zone: &Option<String>) -> MomentoEndpoint {
        MomentoEndpointsResolver::get_data_endpoint_from_hosted_zone(hosted_zone)
            .unwrap_or_else(|| MomentoEndpointsResolver::https_endpoint(claims.c.to_owned()))
    }

    fn hosted_zone_endpoint(hosted_zone: &Option<String>, prefix: &str) -> Option<MomentoEndpoint> {
        if hosted_zone.is_none() {
            return None;
        }
        let hostname = format!("{}{}", prefix, hosted_zone.clone().unwrap());
        Some(MomentoEndpointsResolver::wrapped_endpoint(prefix, hostname, ""))
    }

    fn get_control_endpoint_from_hosted_zone(hosted_zone: &Option<String>) -> Option<MomentoEndpoint> {
        MomentoEndpointsResolver::hosted_zone_endpoint(hosted_zone, CONTROL_ENDPOINT_PREFIX)
    }

    fn get_data_endpoint_from_hosted_zone(hosted_zone: &Option<String>) -> Option<MomentoEndpoint> {
        MomentoEndpointsResolver::hosted_zone_endpoint(hosted_zone, DATA_ENDPOINT_PREFIX)
    }
}
