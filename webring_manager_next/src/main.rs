use async_compat::Compat;
use async_io::block_on;
use lambda_http::{run, service_fn, tower::BoxError};
use webring::*;

fn main() -> Result<(), BoxError> {
    lambda_http::tracing::init_default_subscriber();

    let inc = |x| x + 1;
    let sites = sitelist();

    block_on(Compat::new(run(service_fn(|ev| async {
        build_response(calc_destination(extract_referrer(ev), &sites, inc), &sites)
    }))))
}
