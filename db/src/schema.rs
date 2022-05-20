table! {
    use diesel::sql_types::*;

    advertised_yield (id) {
        id -> Int8,
        vault_address -> Varchar,
        farm_name -> Varchar,
        apr -> Float8,
        scraped_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    deposit_tracking (id) {
        id -> Int8,
        owner_address -> Varchar,
        account_address -> Varchar,
        account_data -> Bytea,
        vault_account_address -> Varchar,
        scraped_at -> Timestamptz,
        current_balance -> Float8,
        current_shares -> Float8,
        balance_usd_value -> Float8,
    }
}

table! {
    use diesel::sql_types::*;

    historic_tshare_price (id) {
        id -> Int8,
        farm_name -> Varchar,
        price -> Float8,
        total_supply -> Float8,
        holder_count -> Float8,
        scraped_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    interest_rate (id) {
        id -> Int8,
        platform -> Varchar,
        asset -> Varchar,
        lending_rate -> Float8,
        borrow_rate -> Float8,
        utilization_rate -> Float8,
        available_amount -> Float8,
        borrowed_amount -> Float8,
        scraped_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    interest_rate_curve (id) {
        id -> Int8,
        platform -> Varchar,
        asset -> Varchar,
        rate_name -> Varchar,
        min_borrow_rate -> Float8,
        max_borrow_rate -> Float8,
        optimal_borrow_rate -> Float8,
        optimal_utilization_rate -> Float8,
        degen_borrow_rate -> Float8,
        degen_utilization_rate -> Float8,
    }
}

table! {
    use diesel::sql_types::*;

    interest_rate_moving_average (id) {
        id -> Int8,
        platform -> Varchar,
        asset -> Varchar,
        rate_name -> Varchar,
        period_start -> Timestamptz,
        period_end -> Timestamptz,
        period_running_average -> Float8,
        period_observed_rates -> Array<Float8>,
        last_period_running_average -> Float8,
    }
}

table! {
    use diesel::sql_types::*;

    lending_optimizer_distribution (id) {
        id -> Int8,
        vault_name -> Varchar,
        standalone_vault_platforms -> Array<Text>,
        standalone_vault_deposited_balances -> Array<Float8>,
    }
}

table! {
    use diesel::sql_types::*;

    realize_yield (id) {
        id -> Int8,
        vault_address -> Varchar,
        farm_name -> Varchar,
        total_deposited_balance -> Float8,
        gain_per_second -> Float8,
        apr -> Float8,
        scraped_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    staking_analytic (id) {
        id -> Int8,
        tokens_staked -> Float8,
        tokens_locked -> Float8,
        stulip_total_supply -> Float8,
        apy -> Float8,
        price_float -> Float8,
        price_uint -> Int8,
        active_unstakes -> Int8,
        scraped_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    token_balance (id) {
        id -> Int8,
        token_account -> Varchar,
        token_mint -> Varchar,
        identifier -> Varchar,
        balance -> Float8,
        scraped_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;

    token_price (id) {
        id -> Int8,
        asset -> Varchar,
        price -> Float8,
        platform -> Varchar,
        coin_in_lp -> Float8,
        pc_in_lp -> Float8,
        asset_identifier -> Varchar,
        period_start -> Timestamptz,
        period_end -> Timestamptz,
        period_observed_prices -> Array<Float8>,
        period_running_average -> Float8,
        last_period_average -> Float8,
        feed_stopped -> Bool,
        token_mint -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    v1_liquidated_position (id) {
        id -> Int8,
        liquidation_event_id -> Varchar,
        temp_liquidation_account -> Varchar,
        authority -> Varchar,
        user_farm -> Varchar,
        obligation -> Varchar,
        started_at -> Timestamptz,
        ended_at -> Nullable<Timestamptz>,
        leveraged_farm -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    v1_obligation_account (id) {
        id -> Int8,
        account -> Varchar,
        authority -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    v1_obligation_ltv (id) {
        id -> Int8,
        authority -> Varchar,
        user_farm -> Varchar,
        account_address -> Varchar,
        ltv -> Float8,
        scraped_at -> Timestamptz,
        leveraged_farm -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    v1_user_farm (id) {
        id -> Int8,
        account_address -> Varchar,
        authority -> Varchar,
        obligations -> Array<Text>,
        obligation_indexes -> Array<Int4>,
        leveraged_farm -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    vault (id) {
        id -> Int8,
        account_address -> Varchar,
        account_data -> Bytea,
        farm_name -> Varchar,
        scraped_at -> Timestamptz,
        last_compound_ts -> Nullable<Timestamptz>,
        last_compound_ts_unix -> Int8,
    }
}

table! {
    use diesel::sql_types::*;

    vault_tvl (id) {
        id -> Int8,
        farm_name -> Varchar,
        total_shares -> Float8,
        total_underlying -> Float8,
        value_locked -> Float8,
        scraped_at -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(
    advertised_yield,
    deposit_tracking,
    historic_tshare_price,
    interest_rate,
    interest_rate_curve,
    interest_rate_moving_average,
    lending_optimizer_distribution,
    realize_yield,
    staking_analytic,
    token_balance,
    token_price,
    v1_liquidated_position,
    v1_obligation_account,
    v1_obligation_ltv,
    v1_user_farm,
    vault,
    vault_tvl,
);
