use crate::jwt::Claims;
use crate::utils;

pub struct MomentoEndpoints {
    pub control_endpoint: String,
    pub data_endpoint: String,
}

pub struct MomentoEndpointsResolver {}

const CONTROL_ENDPOINT_PREFIX: &str = "control.";
const DATA_ENDPOINT_PREFIX: &str = "data.";

impl MomentoEndpointsResolver {
    pub fn resolve(auth_token: &str, hosted_zone: &Option<String>) -> MomentoEndpoints {
        let claims = utils::get_claims(auth_token);
        let control_endpoint = MomentoEndpointsResolver::get_control_endpoint(&claims, hosted_zone);
        let data_endpoint = MomentoEndpointsResolver::get_data_endpoint(&claims, hosted_zone);
        MomentoEndpoints {
            control_endpoint,
            data_endpoint,
        }
    }

    fn get_control_endpoint(claims: &Claims, hosted_zone: &Option<String>) -> String {
        MomentoEndpointsResolver::get_control_endpoint_from_hosted_zone(hosted_zone)
            .unwrap_or_else(|| format!("https://{}:443", claims.cp))
    }

    fn get_data_endpoint(claims: &Claims, hosted_zone: &Option<String>) -> String {
        MomentoEndpointsResolver::get_data_endpoint_from_hosted_zone(hosted_zone)
            .unwrap_or_else(|| format!("https://{}:443", claims.c))
    }

    fn get_control_endpoint_from_hosted_zone(hosted_zone: &Option<String>) -> Option<String> {
        if hosted_zone.is_none() {
            return None;
        }
        return Some(format!(
            "{}{}",
            CONTROL_ENDPOINT_PREFIX,
            hosted_zone.clone().unwrap()
        ));
    }

    fn get_data_endpoint_from_hosted_zone(hosted_zone: &Option<String>) -> Option<String> {
        if hosted_zone.is_none() {
            return None;
        }
        return Some(format!(
            "{}{}",
            DATA_ENDPOINT_PREFIX,
            hosted_zone.clone().unwrap()
        ));
    }
}
