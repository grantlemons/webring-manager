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

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let sites = get_sites();
    let referer = if let Some(referer_header_value) = event.headers().get(REFERER) {
        let referer_string: Uri = referer_header_value.to_str()?.parse()?;
        referer_string.host().unwrap().to_owned()
    } else {
        event
            .query_string_parameters()
            .first("Referer")
            .unwrap_or_default()
            .to_owned()
    };
    let referer_index = sites
        .iter()
        .position(|s| s.parse::<Uri>().unwrap().host().unwrap() == referer)
        .unwrap_or_default() as isize;

    let prev_index = (referer_index - 1).rem_euclid(sites.len() as isize) as usize;
    let prev_site = sites.get(prev_index).unwrap().to_owned();

    let response = Response::builder()
        .header("Location", prev_site)
        .status(StatusCode::SEE_OTHER)
        .body("Referring to next site in webring!".into())?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::http::Request;

    async fn abstraction(referer: &str, location: &str) {
        let request = Request::builder()
            .header(REFERER, format!("{}doesnotexist", referer))
            .body("".into())
            .unwrap();
        assert_eq!(
            function_handler(request)
                .await
                .unwrap()
                .headers()
                .get("Location")
                .unwrap(),
            location
        )
    }

    #[tokio::test]
    async fn test_referer_prev() {
        let sites = [
            "https://lukaswerner.com/",
            "https://grantlemons.com/",
            "https://elijahpotter.dev/",
            "https://b-sharman.dev/",
            "https://lukaswerner.com/",
        ];
        for w in sites.iter().rev().collect::<Vec<_>>().windows(2) {
            abstraction(w[0], w[1]).await
        }
    }
}
