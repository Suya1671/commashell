use serde::Deserialize;
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
    pub async fn get_weather(&self) -> Result<Wttr, reqwest::Error> {
        let response = self.http.get("https://wttr.in/?format=j1").send().await?;
        response.json::<Wttr>().await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wttr {
    current_condition: Vec<CurrentCondition>,
    nearest_area: Vec<NearestArea>,
    request: Vec<Request>,
    weather: Vec<Weather>,
}

impl Wttr {
    pub fn current_condition(&self) -> &CurrentCondition {
        &self.current_condition[0]
    }

    pub fn nearest_area(&self) -> &NearestArea {
        &self.nearest_area[0]
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
            let sunrise = chrono::NaiveTime::parse_from_str(&sunrise, "%I:%M %p").unwrap();
            let sunset = chrono::NaiveTime::parse_from_str(&sunset, "%I:%M %p").unwrap();
            let now = chrono::Local::now().time();

            now < sunrise || now > sunset
        };
        weather_icon_from_wwo_code(&self.current_condition[0].weather_code, night)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct NearestArea {
    area_name: Vec<WeatherDesc>,
    country: Vec<WeatherDesc>,
    latitude: String,
    longitude: String,
    population: String,
    region: Vec<WeatherDesc>,
    weather_url: Vec<WeatherDesc>,
}

impl NearestArea {
    pub fn location(&self) -> &str {
        &self.area_name[0].value
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub query: String,
    #[serde(rename = "type")]
    pub request_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub astronomy: Vec<Astronomy>,
    #[serde(rename = "avgtempC")]
    pub avg_temp_c: String,
    #[serde(rename = "avgtempF")]
    pub avg_temp_f: String,
    pub date: String,
    pub hourly: Vec<Hourly>,
    #[serde(rename = "maxtempC")]
    pub max_temp_c: String,
    #[serde(rename = "maxtempF")]
    pub max_temp_f: String,
    #[serde(rename = "mintempC")]
    pub min_temp_c: String,
    #[serde(rename = "mintempF")]
    pub min_temp_f: String,
    pub sun_hour: String,
    #[serde(rename = "totalSnow_cm")]
    pub total_snow_cm: String,
    pub uv_index: String,
}

#[derive(Debug, Clone, Deserialize)]
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
pub struct Hourly {
    #[serde(rename = "DewPointC")]
    pub dew_point_c: String,
    #[serde(rename = "DewPointF")]
    pub dew_point_f: String,
    #[serde(rename = "FeelsLikeC")]
    pub feels_like_c: String,
    #[serde(rename = "FeelsLikeF")]
    pub feels_like_f: String,
    #[serde(rename = "HeatIndexC")]
    pub heat_index_c: String,
    #[serde(rename = "HeatIndexF")]
    pub heat_index_f: String,
    #[serde(rename = "WindChillC")]
    pub wind_chill_c: String,
    #[serde(rename = "WindChillF")]
    pub wind_chill_f: String,
    #[serde(rename = "WindGustKmph")]
    pub wind_gust_kmph: String,
    #[serde(rename = "WindGustMiles")]
    pub wind_gust_miles: String,
    #[serde(rename = "chanceoffog")]
    pub chance_of_fog: String,
    #[serde(rename = "chanceoffrost")]
    pub chance_of_frost: String,
    #[serde(rename = "chanceofhightemp")]
    pub chance_of_high_temp: String,
    #[serde(rename = "chanceofovercast")]
    pub chance_of_overcast: String,
    #[serde(rename = "chanceofrain")]
    pub chance_of_rain: String,
    #[serde(rename = "chanceofremdry")]
    pub chance_of_remdry: String,
    #[serde(rename = "chanceofsnow")]
    pub chance_of_snow: String,
    #[serde(rename = "chanceofsunshine")]
    pub chance_of_sunshine: String,
    #[serde(rename = "chanceofthunder")]
    pub chance_of_thunder: String,
    #[serde(rename = "chanceofwindy")]
    pub chance_of_windy: String,
    #[serde(rename = "cloudcover")]
    pub cloud_cover: String,
    pub diff_rad: String,
    pub humidity: String,
    pub precip_inches: String,
    #[serde(rename = "precipMM")]
    pub precip_mm: String,
    pub pressure: String,
    pub pressure_inches: String,
    pub short_rad: String,
    pub temp_c: String,
    pub temp_f: String,
    pub time: String,
    pub uv_index: String,
    pub visibility: String,
    pub visibility_miles: String,
    pub weather_code: String,
    pub weather_desc: Vec<WeatherDesc>,
    pub weather_icon_url: Vec<WeatherDesc>,
    pub winddir16_point: String,
    pub winddir_degree: String,
    pub windspeed_kmph: String,
    pub windspeed_miles: String,
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
