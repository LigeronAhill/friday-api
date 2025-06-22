use anyhow::anyhow;

pub fn calculate_coupon(input: &str) -> String {
    if input.is_empty() {
        return String::from("Вы ничего не ввели :(");
    } else if input.split_whitespace().count() != 3 {
        return String::from("Что-то не так с вводом, введите через пробел максимальную длину и ширину помещения, а также ширину рулона (все в метрах)");
    }
    match Measures::try_from(input.to_string()) {
        Ok(measures) => measures.calculate(),
        Err(e) => e.to_string(),
    }
}

struct Measures {
    length: f64,
    width: f64,
    roll_width: f64,
}
impl TryFrom<String> for Measures {
    type Error = anyhow::Error;
    fn try_from(input: String) -> anyhow::Result<Self> {
        let slice = input.split_whitespace().collect::<Vec<&str>>();
        if slice.len() != 3 {
            return Err(anyhow!(
                "Неправильный ввод: введите 'длина ширина ширинарулона' в метрах"
            ));
        }
        let length = slice[0].replace(',', ".").parse::<f64>()?;
        let width = slice[1].replace(',', ".").parse::<f64>()?;
        let roll_width = slice[2].replace(',', ".").parse::<f64>()?;
        Ok(Self {
            length,
            width,
            roll_width,
        })
    }
}
impl Measures {
    fn calculate(&self) -> String {
        let input = format!(
            "Длина помещения: {l:.2} м\nШирина помещения: {w:.2} м\nШирина рулона: {rw:.2} м\n\n",
            l = self.length,
            w = self.width,
            rw = self.roll_width
        );
        let p = self.length * 2.0 + self.width * 2.0;
        let mut k1 = (self.length / self.roll_width).floor() as u16;
        let mut k2 = (self.width / self.roll_width).floor() as u16;
        let o1 = self.length - self.roll_width * (k1 as f64);
        if o1 != 0.0 {
            k1 += 1;
        }
        let o2 = self.width - self.roll_width * (k2 as f64);
        if o2 != 0.0 {
            k2 += 1;
        }
        let s1 = self.width * self.roll_width * (k1 as f64);
        let s2 = self.length * self.roll_width * (k2 as f64);
        let res = if s2 < s1 {
            format!(
                "Площадь покрытия: {s2:.1} м2\nПериметр: {p:.2} м\nОтрезов: {k2} шт\nРазмером: {w:.2} м x {l:.2} м",
                l = self.length,
                w = self.roll_width
            )
        } else {
            format!(
                "Площадь покрытия: {s1:.2} м2\nПериметр: {p:.2} м\nОтрезов: {k1} шт\nРазмером: {w:.2} м x {l:.2} м",
                l = self.width,
                w = self.roll_width
            )
        };
        format!("{input}\n{res}")
    }
}
