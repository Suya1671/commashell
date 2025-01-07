use serde::Deserialize;
#[derive(Default, Debug, Clone)]
pub struct WeatherService {
    http: reqwest::Client,
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    // TODO: allow custom location rather than GeoIP
    pub async fn get_weather(&self, location: &str) -> Result<Wttr, reqwest::Error> {
        let response = self
            .http
            .get(format!("https://wttr.in/{location}?format=j1"))
            .send()
            .await?;
        response.json::<Wttr>().await
    }
}

// I would like to congratulate wttr.in for having the most cursed naming scheme I have ever seen

#[derive(Debug, Clone, Deserialize)]
pub struct Wttr {
    current_condition: Vec<CurrentCondition>,
    request: Vec<Request>,
    weather: Vec<Weather>,
}

impl Wttr {
    pub fn current_condition(&self) -> &CurrentCondition {
        &self.current_condition[0]
    }

    pub fn request(&self) -> &Request {
        &self.request[0]
    }

    pub fn weather(&self) -> &[Weather] {
        &self.weather
    }

    pub fn today_weather(&self) -> &Weather {
        &self.weather[0]
    }

    pub fn temperature_slider(&self) -> TemperatureSlider {
        let current_temp = self.current_condition().temp_c.parse::<f64>().unwrap();
        let min_temp = self.today_weather().min_temp_c.parse::<f64>().unwrap();
        let max_temp = self.today_weather().max_temp_c.parse::<f64>().unwrap();

        TemperatureSlider {
            min: min_temp,
            max: max_temp,
            value: current_temp,
        }
    }

    /// Returns the weather icon name for the current condition
    ///
    /// The icon name is based on the weather code provided by the API
    /// and is mapped to a Gtk icon name.
    ///
    /// # Returns
    /// Returns Ok with the icon name if the weather code is recognized,
    /// otherwise returns Err with the weather code.
    pub fn weather_icon(&self) -> Result<&str, &str> {
        let night = {
            let today_weather = self.today_weather();
            // format: HH:MM AM/PM
            let sunrise = &today_weather.astronomy[0].sunrise;
            let sunset = &today_weather.astronomy[0].sunset;
            // change to 24 hour format
            let sunrise = chrono::NaiveTime::parse_from_str(sunrise, "%I:%M %p").unwrap();
            let sunset = chrono::NaiveTime::parse_from_str(sunset, "%I:%M %p").unwrap();
            let now = chrono::Local::now().time();

            now < sunrise || now > sunset
        };
        weather_icon_from_wwo_code(&self.current_condition[0].weather_code, night)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CurrentCondition {
    #[serde(rename = "FeelsLikeC")]
    feels_like_c: String,
    #[serde(rename = "FeelsLikeF")]
    feels_like_f: String,
    #[serde(rename = "cloudcover")]
    cloud_cover: String,
    humidity: String,
    local_obs_date_time: String,
    #[serde(rename = "observation_time")]
    observation_time: String,
    precip_inches: String,
    #[serde(rename = "precipMM")]
    precip_mm: String,
    pressure: String,
    pressure_inches: String,
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "temp_F")]
    temp_f: String,
    uv_index: String,
    visibility: String,
    visibility_miles: String,
    weather_code: String,
    weather_desc: Vec<WeatherDesc>,
    weather_icon_url: Vec<WeatherDesc>,
    winddir16_point: String,
    winddir_degree: String,
    windspeed_kmph: String,
    windspeed_miles: String,
}

impl CurrentCondition {
    pub fn temperature(&self, unit: TemperatureUnit) -> String {
        match unit {
            TemperatureUnit::Celsius => format!("{}°C", &self.temp_c),
            TemperatureUnit::Fahrenheit => format!("{}°F", &self.temp_f),
        }
    }

    pub fn humidity(&self) -> &str {
        &self.humidity
    }

    /// Returns the cloud cover percentage from 1-100
    pub fn cloud_cover(&self) -> u8 {
        self.cloud_cover.parse().unwrap()
    }

    pub fn uv_index(&self) -> u8 {
        self.uv_index.parse().unwrap()
    }

    pub fn feels_like(&self) -> f64 {
        self.feels_like_c.parse::<f64>().unwrap()
    }

    pub fn desc(&self) -> &str {
        &self.weather_desc[0].value
    }
}

#[derive(Debug)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

#[derive(Debug)]
pub struct TemperatureSlider {
    pub min: f64,
    pub max: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherDesc {
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub query: String,
    #[serde(rename = "type")]
    pub request_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Weather {
    astronomy: Vec<Astronomy>,
    #[serde(rename = "avgtempC")]
    avg_temp_c: String,
    #[serde(rename = "avgtempF")]
    avg_temp_f: String,
    date: String,
    hourly: Vec<Hourly>,
    #[serde(rename = "maxtempC")]
    max_temp_c: String,
    #[serde(rename = "maxtempF")]
    max_temp_f: String,
    #[serde(rename = "mintempC")]
    min_temp_c: String,
    #[serde(rename = "mintempF")]
    min_temp_f: String,
    sun_hour: String,
    #[serde(rename = "totalSnow_cm")]
    total_snow_cm: String,
    uv_index: String,
}

impl Weather {
    pub fn hourly(&self) -> &[Hourly] {
        &self.hourly
    }

    pub fn temperature_slider(&self, unit: TemperatureUnit) -> TemperatureSlider {
        let (current_temp, min_temp, max_temp) = match unit {
            TemperatureUnit::Celsius => (
                self.avg_temp_c.parse::<f64>().unwrap(),
                self.min_temp_c.parse::<f64>().unwrap(),
                self.max_temp_c.parse::<f64>().unwrap(),
            ),
            TemperatureUnit::Fahrenheit => (
                self.avg_temp_f.parse::<f64>().unwrap(),
                self.min_temp_f.parse::<f64>().unwrap(),
                self.max_temp_f.parse::<f64>().unwrap(),
            ),
        };

        TemperatureSlider {
            min: min_temp,
            max: max_temp,
            value: current_temp,
        }
    }

    pub fn desc(&self) -> &str {
        &self.hourly[0].weather_desc[0].value
    }

    pub fn icon(&self) -> Result<&str, &str> {
        let night = false;
        weather_icon_from_wwo_code(&self.hourly[0].weather_code, night)
    }

    pub fn date(&self) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Astronomy {
    pub moon_illumination: String,
    pub moon_phase: String,
    pub moonrise: String,
    pub moonset: String,
    pub sunrise: String,
    pub sunset: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Hourly {
    #[serde(rename = "DewPointC")]
    dew_point_c: String,
    #[serde(rename = "DewPointF")]
    dew_point_f: String,
    #[serde(rename = "FeelsLikeC")]
    feels_like_c: String,
    #[serde(rename = "FeelsLikeF")]
    feels_like_f: String,
    #[serde(rename = "HeatIndexC")]
    heat_index_c: String,
    #[serde(rename = "HeatIndexF")]
    heat_index_f: String,
    #[serde(rename = "WindChillC")]
    wind_chill_c: String,
    #[serde(rename = "WindChillF")]
    wind_chill_f: String,
    #[serde(rename = "WindGustKmph")]
    wind_gust_kmph: String,
    #[serde(rename = "WindGustMiles")]
    wind_gust_miles: String,
    #[serde(rename = "chanceoffog")]
    chance_of_fog: String,
    #[serde(rename = "chanceoffrost")]
    chance_of_frost: String,
    #[serde(rename = "chanceofhightemp")]
    chance_of_high_temp: String,
    #[serde(rename = "chanceofovercast")]
    chance_of_overcast: String,
    #[serde(rename = "chanceofrain")]
    chance_of_rain: String,
    #[serde(rename = "chanceofremdry")]
    chance_of_remdry: String,
    #[serde(rename = "chanceofsnow")]
    chance_of_snow: String,
    #[serde(rename = "chanceofsunshine")]
    chance_of_sunshine: String,
    #[serde(rename = "chanceofthunder")]
    chance_of_thunder: String,
    #[serde(rename = "chanceofwindy")]
    chance_of_windy: String,
    #[serde(rename = "cloudcover")]
    cloud_cover: String,
    diff_rad: String,
    humidity: String,
    precip_inches: String,
    #[serde(rename = "precipMM")]
    precip_mm: String,
    pressure: String,
    pressure_inches: String,
    short_rad: String,
    temp_c: String,
    temp_f: String,
    time: String,
    uv_index: String,
    visibility: String,
    visibility_miles: String,
    weather_code: String,
    weather_desc: Vec<WeatherDesc>,
    weather_icon_url: Vec<WeatherDesc>,
    winddir16_point: String,
    winddir_degree: String,
    windspeed_kmph: String,
    windspeed_miles: String,
}

impl Hourly {
    pub fn temperature(&self, unit: TemperatureUnit) -> String {
        match unit {
            TemperatureUnit::Celsius => format!("{}°C", &self.temp_c),
            TemperatureUnit::Fahrenheit => format!("{}°F", &self.temp_f),
        }
    }

    pub fn temperature_slider(&self, weather: &Weather) -> TemperatureSlider {
        let temp = self.temp_c.parse::<f64>().unwrap();
        let min_temp = weather.min_temp_c.parse::<f64>().unwrap();
        let max_temp = weather.max_temp_c.parse::<f64>().unwrap();

        TemperatureSlider {
            min: min_temp,
            max: max_temp,
            value: temp,
        }
    }

    pub fn humidity(&self) -> &str {
        &self.humidity
    }

    pub fn cloud_cover(&self) -> u8 {
        self.cloud_cover.parse().unwrap()
    }

    pub fn uv_index(&self) -> u8 {
        self.uv_index.parse().unwrap()
    }

    pub fn desc(&self) -> &str {
        self.weather_desc[0].value.trim()
    }

    /// time is millitary style time. E.g. 0 = 12:00 AM, 300 = 3:00 AM, 1200 = 12:00 PM
    pub fn time_str(&self) -> &str {
        &self.time
    }

    pub fn time(&self) -> chrono::NaiveTime {
        let time_str = format!("{:0>4}", self.time_str());
        chrono::NaiveTime::parse_from_str(&time_str, "%H%M").unwrap()
    }

    /// Weather: the weather day this hourly forecast belongs to
    pub fn icon(&self, weather: &Weather) -> Result<&str, &str> {
        let night = {
            // format: HH:MM AM/PM
            let sunrise = &weather.astronomy[0].sunrise;
            let sunset = &weather.astronomy[0].sunset;
            // change to 24 hour format
            let sunrise = chrono::NaiveTime::parse_from_str(sunrise, "%I:%M %p").unwrap();
            let sunset = chrono::NaiveTime::parse_from_str(sunset, "%I:%M %p").unwrap();

            let time = self.time();

            time < sunrise || time > sunset
        };

        weather_icon_from_wwo_code(&self.weather_code, night)
    }
}

fn weather_icon_from_wwo_code(code: &str, night: bool) -> Result<&str, &str> {
    match code {
        "113" => Ok(if night {
            "moon-outline-symbolic"
        } else {
            "sun-outline-symbolic"
        }),
        "116" => Ok(if night {
            "moon-clouds-outline-symbolic"
        } else {
            "few-clouds-outline-symbolic"
        }),
        "119" => Ok("clouds-outline-symbolic"),
        "122" => Ok("clouds-outline-symbolic"),
        "143" => Ok("fog-symbolic"),
        "176" => Ok("rain-symbolic"),
        "179" => Ok("snow-symbolic"),
        "182" => Ok("rain-symbolic"),
        "185" => Ok("rain-symbolic"),
        "200" => Ok("storm-outline-symbolic"),
        "227" => Ok("snow-symbolic"),
        "230" => Ok("snow-symbolic"),
        "248" => Ok("fog-symbolic"),
        "260" => Ok("fog-symbolic"),
        "263" => Ok("rain-symbolic"),
        "266" => Ok("rain-symbolic"),
        "281" => Ok("rain-symbolic"),
        "284" => Ok("rain-symbolic"),
        "293" => Ok("rain-symbolic"),
        "296" => Ok("rain-symbolic"),
        "299" => Ok("rain-symbolic"),
        "302" => Ok("rain-symbolic"),
        "305" => Ok("rain-symbolic"),
        "308" => Ok("rain-symbolic"),
        "311" => Ok("rain-symbolic"),
        "314" => Ok("rain-symbolic"),
        "317" => Ok("snow-symbolic"),
        "320" => Ok("snow-symbolic"),
        "323" => Ok("snow-symbolic"),
        "326" => Ok("snow-symbolic"),
        "329" => Ok("snow-symbolic"),
        "332" => Ok("snow-symbolic"),
        "335" => Ok("snow-symbolic"),
        "338" => Ok("snow-symbolic"),
        "350" => Ok("snow-symbolic"),
        "353" => Ok("rain-symbolic"),
        "356" => Ok("rain-symbolic"),
        "359" => Ok("rain-symbolic"),
        "362" => Ok("snow-symbolic"),
        "365" => Ok("snow-symbolic"),
        "368" => Ok("snow-symbolic"),
        "371" => Ok("snow-symbolic"),
        "374" => Ok("snow-symbolic"),
        "377" => Ok("snow-symbolic"),
        "386" => Ok("storm-outline-symbolic"),
        "389" => Ok("storm-outline-symbolic"),
        "392" => Ok("storm-outline-symbolic"),
        "395" => Ok("storm-outline-symbolic"),
        code => Err(code),
    }
}
