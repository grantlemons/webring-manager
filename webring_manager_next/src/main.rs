use lambda_http::{run, service_fn, tower::BoxError};
use webring::*;

fn main() -> Result<(), BoxError> {
    lambda_http::tracing::init_default_subscriber();

    let inc = |x| x + 1;
    let sites = sitelist();

    smol::block_on(run(service_fn(|ev| {
        build_response(calc_destination(extract_referrer(ev), &sites, inc), &sites)
    })))
}
