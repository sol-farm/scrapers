use anyhow::Result;
use chrono::Utc;

use config::Configuration;
use crossbeam::sync::WaitGroup;

use crossbeam_queue::ArrayQueue;

use diesel::PgConnection;
use log::{debug, error, info};

use std::sync::Arc;

pub fn scrape_interest_rates(config: Arc<Configuration>, conn: &PgConnection) -> Result<()> {
    let db_client = Arc::new(db::client::DBClient {
        conn,
        oob_limit: config.analytics.oob_limit,
    });
    let config = Arc::new(config);
    let rpc_client = Arc::new(config.get_rpc_client(false, None));
    let rate_queue = Arc::new(ArrayQueue::new(
        config.analytics.interest_rates.assets.len(),
    ));
    let wg = WaitGroup::new();
    let rates = config.analytics.interest_rates.assets.clone();
    let start = Utc::now();
    for rate in rates.iter() {
        let rate = rate.clone();
        debug!(
            "looking up interest rate for asset({}) platform({})",
            &rate.asset,
            &rate.platform.to_string()
        );
        let rate_queue = Arc::clone(&rate_queue);
        let rpc_client = Arc::clone(&rpc_client);
        let config = Arc::clone(&config);
        let wg = wg.clone();
        tokio::task::spawn(async move {
            match oracle::rate_lookup::lookup::interest_rate(
                &config,
                &rpc_client,
                &rate.asset,
                &rate.platform.to_string(),
            ) {
                Err(err) => {
                    error!(
                        "failed to lookup interest rate for {}: {:#?}",
                        &rate.asset, err
                    );
                }
                Ok(price) => {
                    if rate_queue.push(price).is_err() {
                        error!("failed to push price record into queue");
                    }
                }
            }
            drop(wg);
        });
    }
    info!("waiting for interest rate lookup routines to finish");
    wg.wait();
    info!("interest rate lookup routines finished, storing results");
    loop {
        match rate_queue.pop() {
            Some(record) => {
                if let Err(err) = db_client.put_interest_rate(
                    record.platform.clone(),
                    record.asset.clone(),
                    record.rate,
                    record.utilization_rate,
                    record.interest_rate,
                    record.available_amount,
                    record.borrowed_amount,
                    start,
                ) {
                    error!(
                        "failed to update interest rate for asset({}) platform({}): {:#?}",
                        record.asset, record.platform, err
                    );
                }
            }
            None => {
                info!("no more interest rates to observe");
                break;
            }
        }
    }
    let end = Utc::now();
    let diff = end.signed_duration_since(start);
    info!(
        "total time to scrape and update interest rate records {} seconds",
        diff.num_seconds()
    );
    Ok(())
}
