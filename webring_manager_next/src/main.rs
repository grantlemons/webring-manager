use lambda_http::{
    http::{header::REFERER, StatusCode, Uri},
    run, service_fn, Body, Error, Request, RequestExt, Response,
};

fn get_sites() -> Vec<String> {
    std::env::var("SITES")
        .unwrap()
        .split(", ")
        .map(|s| s.to_owned())
        .collect()
}

async fn function_handler(sites: Vec<String>, event: Request) -> Result<Response<Body>, Error> {
    let referer = if let Some(referer_header_value) = event.headers().get(REFERER) {
        referer_header_value.to_str()?.to_owned()
    } else {
        event
            .query_string_parameters()
            .first("Referer")
            .unwrap_or_default()
            .to_owned()
    }
    .parse::<Uri>()?
    .host()
    .unwrap()
    .to_owned();

    let referer_index = sites
        .iter()
        .position(|s| s.parse::<Uri>().unwrap().host().unwrap() == referer)
        .unwrap_or_default() as isize;

    let next_index = (referer_index + 1).rem_euclid(sites.len() as isize) as usize;
    let next_site = sites.get(next_index).unwrap().to_owned();

    let response = Response::builder()
        .header("Location", next_site)
        .status(StatusCode::SEE_OTHER)
        .body("Referring to next site in webring!".into())?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(|ev| function_handler(get_sites(), ev))).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::http::Request;

    async fn abstraction(sites: &[String], referer: &str, location: &str) {
        let request = Request::builder()
            .header(REFERER, referer)
            .body("".into())
            .unwrap();
        assert_eq!(
            function_handler(sites.to_vec(), request)
                .await
                .unwrap()
                .headers()
                .get("Location")
                .unwrap(),
            location
        )
    }

    #[tokio::test]
    async fn test_referer_next() {
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
            abstraction(&sites, &w[0], &w[1]).await
        }
    }
}
