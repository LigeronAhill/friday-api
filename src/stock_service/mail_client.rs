use chrono::TimeZone;
use mail_parser::MimeHeaders;
use tracing::error;

use crate::Result;

use super::FetchMap;
const QUERY: &str = "RFC822";
const INBOX: &str = "INBOX";

#[derive(Clone)]
pub struct MailClient {
    user: String,
    pass: String,
    host: String,
    from: usize,
}
impl MailClient {
    pub fn new(user: String, pass: String, host: String) -> Result<MailClient> {
        let mut mail_client = MailClient {
            user,
            pass,
            host,
            from: 0,
        };
        let mut session = mail_client.session()?;
        session.select(INBOX)?;
        let msg_count = session.search("ALL")?.len();
        mail_client.from = msg_count - 300;
        session.logout()?;
        Ok(mail_client)
    }
    fn session(&self) -> Result<imap::Session<Box<dyn imap::ImapConnection>>> {
        let client = imap::ClientBuilder::new(&self.host, 993)
            .danger_skip_tls_verify(true)
            .connect()?;
        let mut session = client.login(&self.user, &self.pass).map_err(|e| e.0)?;
        session.select(INBOX)?;
        Ok(session)
    }
    pub fn fetch(&mut self) -> Result<FetchMap> {
        let mut supmap = std::collections::HashMap::new();
        supmap.insert("vvolodin@opuscontract.ru", "opus");
        supmap.insert("sales@bratec-lis.com", "fox");
        supmap.insert("rassilka@fancyfloor.ru", "fancy");
        // supmap.insert("sale8@fancy-floor.ru", "fancy");
        supmap.insert("ulyana.boyko@carpetland.ru", "carpetland");
        supmap.insert("dealer@kover-zefir.ru", "zefir");
        supmap.insert("almaz2008@yandex.ru", "fenix");
        let mut session = self.session()?;
        let msg_count = session.search("ALL")?.len();
        let q = format!("{}:{msg_count}", self.from);
        tracing::info!("Получаю письма с {} по {msg_count}", self.from);
        let fetches = session.fetch(q, QUERY)?;
        self.from = msg_count;
        let mut m = std::collections::HashMap::new();
        for fetch in fetches.iter() {
            let fetch_date = fetch.internal_date().map(|d| d.to_utc());
            if let Some(body) = fetch.body() {
                if let Some(parsed) = mail_parser::MessageParser::default().parse(body) {
                    let sender = parsed
                        .from()
                        .and_then(|a| a.first().and_then(|s| s.address()))
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    if let Some(supplier) = supmap.get(sender.as_str()) {
                        let attachments = parsed
                            .attachments()
                            .flat_map(|a| {
                                if a.attachment_name().is_some_and(|n| {
                                    n.to_lowercase().contains("склад")
                                        || n.to_lowercase().contains("остат")
                                }) || (sender == "vvolodin@opuscontract.ru"
                                    && a.attachment_name().is_none())
                                {
                                    Some(a.contents().to_vec())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if !attachments.is_empty() {
                            let received_date =
                                parsed.received().and_then(|r| r.date()).and_then(|d| {
                                    chrono::Utc
                                        .with_ymd_and_hms(
                                            d.year as i32,
                                            d.month as u32,
                                            d.day as u32,
                                            d.hour as u32,
                                            d.minute as u32,
                                            d.second as u32,
                                        )
                                        .single()
                                });
                            let date = parsed.date().and_then(|d| {
                                chrono::Utc
                                    .with_ymd_and_hms(
                                        d.year as i32,
                                        d.month as u32,
                                        d.day as u32,
                                        d.hour as u32,
                                        d.minute as u32,
                                        d.second as u32,
                                    )
                                    .single()
                            });
                            let received = if let Some(r) = date {
                                r
                            } else if let Some(d) = received_date {
                                d
                            } else if let Some(d) = fetch_date {
                                d
                            } else {
                                error!("Не получилось прочитать дату письма {supplier}");
                                chrono::Utc::now()
                            };
                            m.insert(supplier.to_string(), (attachments, received));
                        }
                    }
                }
            }
        }
        session.logout()?;
        Ok(m)
    }
}
