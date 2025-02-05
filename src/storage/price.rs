// use crate::{models::Price, Result};
//
// #[derive(Clone)]
// pub struct PriceStorage {
//     pool: sqlx::PgPool,
// }
//
// impl PriceStorage {
//     pub fn new(pool: sqlx::PgPool) -> PriceStorage {
//         PriceStorage { pool }
//     }
//     pub async fn update(&self, input: Vec<Price>) -> Result<u64> {
//         let query_string = "INSERT INTO prices(supplier, product_type, brand, name, purchase_price, purchase_price_currency, recommended_price, recommended_price_currency, colors, widths) ";
//         let mut query_builder = sqlx::QueryBuilder::new(query_string);
//         query_builder.push_values(input, |mut b, price| {
//             b.push_bind(price.supplier)
//                 .push_bind(price.product_type)
//                 .push_bind(price.brand)
//                 .push_bind(price.name)
//                 .push_bind(price.purchase_price)
//                 .push_bind(price.purchase_price_currency)
//                 .push_bind(price.recommended_price)
//                 .push_bind(price.recommended_price_currency)
//                 .push_bind(price.colors)
//                 .push_bind(price.widths);
//         });
//         query_builder.push(" ON CONFLICT(u_constraint) DO UPDATE SET purchase_price = EXCLUDED.purchase_price, purchase_price_currency = EXCLUDED.purchase_price_currency, recommended_price = EXCLUDED.recommended_price, recommended_price_currency = EXCLUDED.recommended_price_currency, colors = EXCLUDED.colors, widths = EXCLUDED.widths, updated = now();");
//         let query = query_builder.build();
//         let results = query.execute(&self.pool).await?;
//         let rows_affected = results.rows_affected();
//         Ok(rows_affected)
//     }
//     pub async fn get(&self) -> Result<Vec<Price>> {
//         let query = "SELECT * FROM price";
//         let results = sqlx::query_as::<_, Price>(query).fetch_all(&self.pool).await?;
//         Ok(results)
//     }
//     pub async fn find(&self, search_string: String) -> Result<Vec<Price>> {
//         let query = "SELECT * FROM price WHERE name = $1";
//         let results = sqlx::query_as::<_, Price>(query).bind(format!("%{search_string}%")).fetch_all(&self.pool).await?;
//         Ok(results)
//     }
// }

