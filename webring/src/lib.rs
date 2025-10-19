use lambda_http::{
    Body, Request, RequestExt, Response,
    http::{StatusCode, Uri, header::REFERER},
    tower::BoxError,
};

pub fn sitelist() -> Vec<String> {
    std::env::var("SITES")
        .unwrap()
        .split(" ")
        .map(str::to_owned)
        .collect()
}

fn parse_sites(sites: &[String]) -> Vec<(String, &str)> {
    sites
        .iter()
        .filter_map(|s| Some((parse_uri_host(s).ok()?, s.as_str())))
        .collect()
}

fn parse_uri_host(s: impl Into<String>) -> Result<String, String> {
    s.into()
        .parse::<Uri>()
        .map(|v| {
            v.host()
                .map(str::to_owned)
                .ok_or("No host in referer".to_owned())
        })
        .map_err(|_| "Invalid uri")?
}

pub fn extract_referrer(req: Request) -> Result<String, String> {
    let parameters = req.query_string_parameters();
    let query_parameter_referer_host = parameters
        .first("Referer")
        .or(parameters.first("referer"))
        .map(str::to_owned)
        .ok_or("No Referer query parameter".to_owned())
        .and_then(parse_uri_host);
    let header_referer_host = req
        .headers()
        .get(REFERER)
        .ok_or("No referer header".to_owned())
        .and_then(|hv| {
            hv.to_str()
                .map(str::to_owned)
                .map_err(|_| "Header value not ASCII".to_owned())
        })
        .and_then(parse_uri_host);

    query_parameter_referer_host
        .or(header_referer_host.map_err(|_| "No referer header or query parameter".to_owned()))
}

pub fn calc_destination<F: Fn(isize) -> isize>(
    referer: Result<String, String>,
    sites: &[String],
    f: F,
) -> Result<String, String> {
    let referer = referer?;
    let hosts = parse_sites(sites);
    let referer_index = hosts
        .iter()
        .position(|(h, _)| *h == referer)
        .ok_or("Referer not in hosts list!".to_owned())? as isize;

    let next_index = f(referer_index).rem_euclid(hosts.len() as isize) as usize;
    hosts
        .get(next_index)
        .map(|(_, u)| u.to_string())
        .ok_or("No next site".to_owned())
}

pub fn build_response(
    site: Result<String, String>,
    sites: &[String],
) -> Result<Response<Body>, BoxError> {
    Ok(match site {
        Ok(site) => Response::builder()
            .header("Location", &site)
            .status(StatusCode::SEE_OTHER)
            .body(
                format!(
                    "Referring to {site}\nFull site list: {:#?}",
                    parse_sites(sites)
                        .iter()
                        .map(|(_, h)| h)
                        .collect::<Vec<_>>()
                )
                .into(),
            )?,
        Err(e) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(e.into())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::http::Request;

    fn abstraction(sites: &[String], referer: &str, location: &str) {
        let request = Request::builder()
            .header(REFERER, referer)
            .body("".into())
            .unwrap();
        assert_eq!(
            build_response(
                calc_destination(extract_referrer(request), &sites, |x| x + 1),
                &sites
            )
            .unwrap()
            .headers()
            .get("Location")
            .unwrap(),
            location
        )
    }

    #[test]
    fn windows() {
        let sites = [
            "https://lukaswerner.com/",
            "https://grantlemons.com/",
            "https://elijahpotter.dev/",
            "https://b-sharman.dev/",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        for w in sites.windows(2) {
            abstraction(&sites, &w[0], &w[1])
        }
    }

    #[test]
    fn bad_uri_filtered() {
        let sites = [
            "https:/lukaswerner.com/",
            "https://grantlemons.com/",
            "https://elijahpotter.dev/",
            "https://b-sharman.dev/",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        abstraction(&sites, &sites[3], &sites[1])
    }
}
