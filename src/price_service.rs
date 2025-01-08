use crate::models::{Currency, Price, PriceItem};
use crate::Result;

#[derive(Clone)]
pub struct PriceService {
    storage: crate::storage::PriceStorage,
}

impl PriceService {
    pub fn new(storage: crate::storage::PriceStorage) -> Self {
        PriceService { storage }
    }
    pub async fn update(&self, input: Vec<PriceItem>, currencies: Vec<Currency>) -> Result<u64> {
        self.storage.update(convert(input, currencies)).await
    }
    pub async fn get(&self) -> Result<Vec<Price>> {
        self.storage.get().await
    }
    pub async fn find(&self, search_string: String) -> Result<Vec<Price>> {
        self.storage.find(search_string).await
    }
}

fn convert(input: Vec<PriceItem>, currencies: Vec<Currency>) -> Vec<Price> {
    let mut result: Vec<Price> = Vec::new();
    for item in input {
        if let Some(ppc) = currencies.iter().find(|c| c.char_code == item.purchase_price_currency) {
            let price = Price {
                id: uuid::Uuid::new_v4(),
                supplier: item.supplier,
                product_type: item.product_type,
                brand: item.brand,
                name: item.name,
                purchase_price: item.purchase_price,
                purchase_price_currency: ppc.id,
                recommended_price: item.recommended_price,
                recommended_price_currency: currencies.iter().find(|c| item.recommended_price_currency.clone().is_some_and(|p| p == c.char_code)).map(|d| d.id),
                colors: item.colors,
                widths: item.widths,
                updated: chrono::Utc::now(),
            };
            result.push(price);
        }
    }
    result
}