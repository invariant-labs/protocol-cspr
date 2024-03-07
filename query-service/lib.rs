use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use invariant::InvariantRef;
use odra::types::Address;
use odra::types::U128;
use std::str::FromStr;

pub mod structs;

use structs::FeeTierExistParams;

pub async fn fee_tier_exist(params: web::Json<FeeTierExistParams>) -> impl Responder {
    let address = Address::from_str(&params.address).unwrap();
    let fee = U128::from(params.fee);
    let tick_spacing = params.tick_spacing as u32;
    println!(
        "Received parameters: address={:?}, fee={}, tick_spacing={}",
        address, fee, tick_spacing
    );

    let result = web::block(move || {
        let invariant = InvariantRef::at(&address);
        let r = invariant.fee_tier_return();
        println!("fee_tier_return: {:?}", r);
        r
    })
    .await;

    println!("result: {:?}", result);

    HttpResponse::Ok().body(format!(
        "Received parameters: address={:?}, fee={}, tick_spacing={}",
        address, fee, tick_spacing
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        // TODO: CORS
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(web::resource("/fee_tier_exist").route(web::post().to(fee_tier_exist)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
