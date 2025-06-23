use crate::models::{Price, PriceItem};

#[derive(Clone)]
pub struct PriceStorage {
    pool: sqlx::PgPool,
}
impl PriceStorage {
    pub fn new(pool: sqlx::PgPool) -> Self {
        PriceStorage { pool }
    }
    pub async fn get(&self, limit: i32, offset: i32) -> Result<Vec<Price>, sqlx::Error> {
        sqlx::query_as::<_, Price>("SELECT * FROM prices ORDER BY name LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
    }
    pub async fn get_by_supplier(&self, supplier: &str) -> Result<Vec<Price>, sqlx::Error> {
        sqlx::query_as::<_, Price>("SELECT * FROM prices WHERE supplier = $1")
            .bind(supplier.to_uppercase())
            .fetch_all(&self.pool)
            .await
    }
    pub async fn find(&self, search_string: &str) -> Result<Vec<Price>, sqlx::Error> {
        let mut re = String::from(".*");
        for word in search_string.split_whitespace() {
            re.push_str(word);
            re.push_str(".*");
        }
        let query = "SELECT * FROM prices WHERE name ~* $1 LIMIT 100";
        sqlx::query_as::<_, Price>(query)
            .bind(re)
            .fetch_all(&self.pool)
            .await
    }
    pub async fn update(&self, input: Vec<PriceItem>) -> Result<u64, sqlx::Error> {
        let query_string = r#"
        INSERT INTO prices (supplier, manufacturer, collection, name, widths, pile_composition, pile_height, total_height, pile_weight, total_weight, durability_class, fire_certificate, purchase_roll_price, purchase_coupon_price, recommended_roll_price, recommended_coupon_price) 
        "#;
        let mut query_builder = sqlx::QueryBuilder::new(query_string);
        query_builder.push_values(input, |mut b, input| {
            b.push_bind(input.supplier)
                .push_bind(input.manufacturer)
                .push_bind(input.collection)
                .push_bind(input.name)
                .push_bind(input.widths)
                .push_bind(input.pile_composition)
                .push_bind(input.pile_height)
                .push_bind(input.total_height)
                .push_bind(input.pile_weight)
                .push_bind(input.total_weight)
                .push_bind(input.durability_class)
                .push_bind(input.fire_certificate)
                .push_bind(input.purchase_roll_price)
                .push_bind(input.purchase_coupon_price)
                .push_bind(input.recommended_roll_price)
                .push_bind(input.recommended_coupon_price);
        });
        let conflict = " ON CONFLICT(supplier, manufacturer, collection) DO UPDATE SET widths = EXCLUDED.widths,
        pile_composition = EXCLUDED.pile_composition,
        pile_height = EXCLUDED.pile_height,
        total_height = EXCLUDED.total_height,
        pile_weight = EXCLUDED.pile_weight,
        total_weight = EXCLUDED.total_weight,
        durability_class = EXCLUDED.durability_class,
        fire_certificate = EXCLUDED.fire_certificate,
        purchase_roll_price = EXCLUDED.purchase_roll_price,
        purchase_coupon_price = EXCLUDED.purchase_coupon_price,
        recommended_roll_price = EXCLUDED.recommended_roll_price,
        recommended_coupon_price = EXCLUDED.recommended_coupon_price,
        updated = now();";
        query_builder.push(conflict);
        let query = query_builder.build();
        let results = query.execute(&self.pool).await?;
        let inserted = results.rows_affected();
        Ok(inserted)
    }
}
