use lambda_http::{run, service_fn, Error, IntoResponse, Request};

async fn function_handler(event: Request) -> Result<impl IntoResponse, std::convert::Infallible> {
    // Extract some useful information from the request
    println!("event data payload: {:?}", event);
    Result::<&str, std::convert::Infallible>::Ok("end of lambda")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await?;
    Ok(())
}
