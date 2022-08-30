//! # `teleinfo-nom`
//! A lib to parse teleinfo (french power provider)

extern crate chrono;
extern crate nom;

use chrono::{offset::Local, DateTime};
use std::collections::HashMap;
use std::io::{self, Error, ErrorKind, Read, Result};

type TeleinfoTuple<'a> = (&'a str, &'a str, char, Option<TeleinfoDate>);

/// Describes the mode of a Teleinfo message
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TeleinfoMode {
    Standard,
    Legacy,
}

/// TeleinfoDate struct represents a date sent in a teleinfo message in standard mode
#[derive(Clone, Debug, PartialEq)]
pub struct TeleinfoDate {
    /// char representing the season might by 'h', 'e', or ' ' upper or lower case depending
    /// whether meter clock is synchronized or not
    pub season: char,
    /// the DateTime parsed from Teleinfo message
    pub date: DateTime<Local>,
    pub raw_value: String,
}

/// TeleinfoValue represents the value and date of a message line from Teleinfo
#[derive(Clone, Debug, PartialEq)]
pub struct TeleinfoValue {
    pub value: String,
    pub horodate: Option<TeleinfoDate>,
}

/// TeleinfoMessageType describes if the message is a short message or a normal message
#[derive(Debug, PartialEq)]
pub enum TeleinfoMessageType {
    Short,
    Normal,
}

/// TeleinfoMeterType describes if the meter is monophasé or triphase
#[derive(Debug, PartialEq)]
pub enum TeleinfoMeterType {
    MonoPhase,
    TriPhase,
}

/// Representation of a full message from teleinfo
/// * values is hashmap resolving index to TeleinfoValue
/// * mode the mode of the messae as TeleinfoMode
/// * valid whether the message is valid checksum wise
#[derive(Clone, Debug, PartialEq)]
pub struct TeleinfoMessage {
    values: HashMap<String, TeleinfoValue>,
    mode: TeleinfoMode,
    valid: bool,
}

impl TeleinfoMessage {
    /// Return message type as `TeleinfoMessageType`
    /// # Example
    /// ```
    /// use std::fs::File;
    /// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_message_type(),teleinfo_nom::TeleinfoMessageType::Normal)
    /// ```
    pub fn get_message_type(&self) -> TeleinfoMessageType {
        match self.mode {
            TeleinfoMode::Standard => TeleinfoMessageType::Normal,
            TeleinfoMode::Legacy => {
                if self.values.contains_key("OPTARIF") {
                    TeleinfoMessageType::Normal
                } else {
                    TeleinfoMessageType::Short
                }
            }
        }
    }

    /// Return meter type as `TeleinfoMeterType`
    /// # Example
    /// ```
    /// use std::fs::File;
    /// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_meter_type(),teleinfo_nom::TeleinfoMeterType::TriPhase)
    /// ```
    ///
    pub fn get_meter_type(&self) -> TeleinfoMeterType {
        match self.mode {
            TeleinfoMode::Legacy => {
                if self.values.contains_key("IINST1") {
                    TeleinfoMeterType::TriPhase
                } else {
                    TeleinfoMeterType::MonoPhase
                }
            }
            TeleinfoMode::Standard => {
                if self.values.contains_key("SINSTS1") {
                    TeleinfoMeterType::TriPhase
                } else {
                    TeleinfoMeterType::MonoPhase
                }
            }
        }
    }

    /// Return the index currently increasing
    ///
    /// # Example
    /// ```
    /// use std::fs::File;
    /// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_current_index(),"EASF03".to_string());
    /// let mut stream = File::open("assets/stream_legacy_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_current_index(),"BBRHPJB".to_string())
    /// ```
    pub fn get_current_index(&self) -> String {
        match self.mode {
            TeleinfoMode::Legacy => self.get_current_index_legacy(),
            TeleinfoMode::Standard => self.get_current_index_standard(),
        }
    }

    fn get_current_index_legacy(&self) -> String {
        match self.get_value("PTEC".to_string()).unwrap().value.as_str() {
            "TH.." => "BASE".to_string(),
            "HC.." => "HCHC".to_string(),
            "HP.." => "HCHP".to_string(),
            "HN.." => "EJPHN".to_string(),
            "PM.." => "EJPPM".to_string(),
            "HCJB" => "BBRHCJB".to_string(),
            "HCJW" => "BBRHCJW".to_string(),
            "HCJR" => "BBRHCJR".to_string(),
            "HPJB" => "BBRHPJB".to_string(),
            "HPJW" => "BBRHPJW".to_string(),
            "HPJR" => "BBRHPJR".to_string(),
            &_ => "BASE".to_string(),
        }
    }

    fn get_current_index_standard(&self) -> String {
        let idx = &self.get_value("NTARF".to_string()).unwrap().value;
        format!("EASF{}", idx)
    }

    /// Return all relevant billing indices for the message
    /// # Example
    /// ```
    /// use std::fs::File;
    /// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_billing_indices(),vec![
    ///        "EASF01".to_string(),
    ///        "EASF02".to_string(),
    ///        "EASF03".to_string(),
    ///        "EASF04".to_string(),
    ///        "EASF05".to_string(),
    ///        "EASF06".to_string(),
    ///        "EASF07".to_string(),
    ///        "EASF08".to_string(),
    ///        "EASF09".to_string(),
    ///        "EASF10".to_string(),
    ///    ]);
    /// let mut stream = File::open("assets/stream_legacy_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_billing_indices(),vec![
    /// "BBRHCJB".to_string(),
    /// "BBRHPJB".to_string(),
    /// "BBRHCJR".to_string(),
    /// "BBRHPJR".to_string(),
    /// "BBRHCJW".to_string(),
    /// "BBRHPJW".to_string(),
    /// ])
    /// ```
    pub fn get_billing_indices(&self) -> Vec<String> {
        match self.mode {
            TeleinfoMode::Legacy => self.get_billing_indices_legacy(),
            TeleinfoMode::Standard => self.get_billing_indices_standard(),
        }
    }

    fn get_billing_indices_legacy(&self) -> Vec<String> {
        let mut optarif = self
            .get_value("OPTARIF".to_string())
            .unwrap()
            .value
            .as_str();
        if &optarif[0..3] == "BBR" {
            optarif = "BBR";
        }
        match optarif {
            "BASE" => vec!["BASE".to_string()],
            "HC.." => vec!["HCHC".to_string(), "HCHP".to_string()],
            "EJP." => vec!["EJPHN".to_string(), "EJPPM".to_string()],
            "BBR" => vec![
                "BBRHCJB".to_string(),
                "BBRHPJB".to_string(),
                "BBRHCJR".to_string(),
                "BBRHPJR".to_string(),
                "BBRHCJW".to_string(),
                "BBRHPJW".to_string(),
            ],
            &_ => vec!["BASE".to_string()],
        }
    }

    fn get_billing_indices_standard(&self) -> Vec<String> {
        vec![
            "EASF01".to_string(),
            "EASF02".to_string(),
            "EASF03".to_string(),
            "EASF04".to_string(),
            "EASF05".to_string(),
            "EASF06".to_string(),
            "EASF07".to_string(),
            "EASF08".to_string(),
            "EASF09".to_string(),
            "EASF10".to_string(),
        ]
    }

    /// Return a &TeleinfoValue as Option for `key`
    /// # Example
    /// ```
    /// use std::fs::File;
    /// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_value("EASF03".to_string()).unwrap().value,"000487131");
    /// let mut stream = File::open("assets/stream_legacy_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_value("BBRHPJB".to_string()),Some(&teleinfo_nom::TeleinfoValue{value: "001012295".to_string(),
    /// horodate: None }))
    /// ```
    pub fn get_value(&self, key: String) -> Option<&TeleinfoValue> {
        self.values.get(&key)
    }

    /// Return a vector of tuples with (index,Option(value)) from a vector of indices to fetch
    /// # Example
    /// ```
    /// use std::fs::File;
    /// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_values(result.get_billing_indices()),
    ///            vec![
    ///            ("EASF01".to_string(),Some("004855593".to_string())),
    ///            ("EASF02".to_string(),Some("014090959".to_string())),
    ///            ("EASF03".to_string(),Some("000487131".to_string())),
    ///            ("EASF04".to_string(),Some("001481464".to_string())),
    ///            ("EASF05".to_string(),Some("000227596".to_string())),
    ///            ("EASF06".to_string(),Some("000706363".to_string())),
    ///            ("EASF07".to_string(),Some("000000000".to_string())),
    ///            ("EASF08".to_string(),Some("000000000".to_string())),
    ///            ("EASF09".to_string(),Some("000000000".to_string())),
    ///            ("EASF10".to_string(),Some("000000000".to_string())),
    ///            ]);
    /// let mut stream = File::open("assets/stream_legacy_raw.txt").unwrap();
    /// let (remain, result) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
    /// assert_eq!(result.get_values(result.get_billing_indices()),
    ///            vec![
    ///            ("BBRHCJB".to_string(),Some("001478389".to_string())),
    ///            ("BBRHPJB".to_string(),Some("001012295".to_string())),
    ///            ("BBRHCJR".to_string(),Some("000025098".to_string())),
    ///            ("BBRHPJR".to_string(),Some("000006010".to_string())),
    ///            ("BBRHCJW".to_string(),Some("000134553".to_string())),
    ///            ("BBRHPJW".to_string(),Some("000213701".to_string())),
    ///            ]);
    pub fn get_values(&self, keys: Vec<String>) -> Vec<(String, Option<String>)> {
        keys.into_iter()
            .map(|idx| (idx.clone(), get_value_from_teleinfovalue(self.get_value(idx))))
            .collect()
    }
}

pub mod parser;

fn get_value_from_teleinfovalue(value: Option<&TeleinfoValue>) -> Option<String> {
    match value {
        Some(x) => Some(x.value.clone()),
        None => None,
    }
}

fn parsed_vector_to_values(lines: Vec<TeleinfoTuple>) -> HashMap<String, TeleinfoValue> {
    let mut values = HashMap::new();
    for line in lines {
        match line {
            (key, val, _, hd) => values.insert(
                key.to_string(),
                TeleinfoValue {
                    value: val.to_string(),
                    horodate: hd,
                },
            ),
        };
    }
    values
}

fn build_message(raw_message: &str) -> Result<TeleinfoMessage> {
    let (r, (lines, mode)) = parser::parser_message(raw_message).unwrap();
    let mut result = TeleinfoMessage {
        values: HashMap::new(),
        mode,
        valid: true,
    };
    result.valid = r.is_empty() && parser::validate_message(mode, lines.clone());
    result.values = parsed_vector_to_values(lines);
    Ok(result)
}

/// Read message from an readable object `source`, with `leftover` being the unparsed string
/// from a previous call
/// Returns a tuple with to be parsed in a next call string as `leftover` and the first found TeleinfoMessage
/// # Example
/// ```
/// use std::fs::File;
/// // Could be a serial port with serialport crate
/// let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
/// let (remain, msg1) = teleinfo_nom::get_message(&mut stream, "".to_string()).unwrap();
/// ```
pub fn get_message<T: Read>(source: &mut T, leftover: String) -> Result<(String, TeleinfoMessage)> {
    let mut acc: Vec<u8> = Vec::with_capacity(2000);
    //let mut buf: Vec<u8> = Vec::with_capacity(200);
    let mut leftover = leftover.as_bytes().to_vec();
    acc.append(&mut leftover);
    loop {
        let mut buf: Vec<u8> = vec![0; 200];
        buf = match source.read(buf.as_mut_slice()) {
            Ok(t) => buf[..t].to_vec(),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => vec![],
            Err(e) => return Err(e),
        };
        acc.append(&mut buf);
        let current_clone = acc.clone();
        let current_data = String::from_utf8_lossy(&current_clone);
        match parser::get_message(&current_data) {
            Ok((r, message)) => {
                let result = build_message(message).unwrap();
                return Ok((r.to_string(), result));
            }
            Err(nom::Err::Incomplete(_)) => (),
            Err(_) => return handle_nom_error(),
        };
    }
}

fn handle_nom_error() -> Result<(String, TeleinfoMessage)> {
    Err(Error::new(ErrorKind::InvalidData, "Parse Error"))
}

#[cfg(test)]
mod tests {
    use crate::get_message;
    use crate::parsed_vector_to_values;
    use crate::TeleinfoDate;
    use crate::TeleinfoMessage;
    use crate::TeleinfoMode;
    use chrono::{Local, TimeZone};
    use std::fs::File;
    #[test]
    fn test_get_message() {
        let mut stream = File::open("assets/stream_standard_raw.txt").unwrap();
        let expect_values = vec![
            ("ADSC","041776199277",'I',None),
            ("VTIC","02",'J',None),
            ("DATE","",';',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(23, 8, 4), raw_value: "H200214230804".to_string() })),
            ("NGTF","     TEMPO      ",'F',None),
            ("LTARF","   HC  BLANC    ",'6',None),
            ("EAST","021849106",'.',None),
            ("EASF01","004855593",'I',None),
            ("EASF02","014090959",'H',None),
            ("EASF03","000487131",'<',None),
            ("EASF04","001481464",'A',None),
            ("EASF05","000227596",'E',None),
            ("EASF06","000706363",'@',None),
            ("EASF07","000000000",'(',None),
            ("EASF08","000000000",')',None),
            ("EASF09","000000000",'*',None),
            ("EASF10","000000000",'"',None),
            ("EASD01","021849106",'?',None),
            ("EASD02","000000000",'!',None),
            ("EASD03","000000000",'"',None),
            ("EASD04","000000000",'#',None),
            ("IRMS1","003",'1',None),
            ("IRMS2","006",'5',None),
            ("IRMS3","003",'3',None),
            ("URMS1","237",'F',None),
            ("URMS2","238",'H',None),
            ("URMS3","235",'F',None),
            ("PREF","30",'B',None),
            ("PCOUP","30",'\\',None),
            ("SINSTS","02700",'O',None),
            ("SINSTS1","00664",'G',None),
            ("SINSTS2","01373",'F',None),
            ("SINSTS3","00664",'I',None),
            ("SMAXSN","10802",'7',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(17, 51, 35), raw_value: "H200214175135".to_string() })),
            ("SMAXSN1","03411",'&',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(17, 51, 35), raw_value: "H200214175135".to_string() })),
            ("SMAXSN2","03899",';',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(17, 51, 35), raw_value: "H200214175135".to_string() })),
            ("SMAXSN3","03512",'*',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(17, 51, 35), raw_value: "H200214175135".to_string() })),
            ("SMAXSN-1","09562",' ',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 13).and_hms(8, 51, 18), raw_value: "H200213085118".to_string() })),
            ("SMAXSN1-1","03129",'J',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 13).and_hms(8, 51, 18), raw_value: "H200213085118".to_string() })),
            ("SMAXSN2-1","03366",'@',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 13).and_hms(10, 11, 42), raw_value: "H200213101142".to_string() })),
            ("SMAXSN3-1","03191",'K',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 13).and_hms(8, 51, 18), raw_value: "H200213085118".to_string() })), 
            ("CCASN","01650",'5',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(23, 0, 0), raw_value: "H200214230000".to_string() })),
            ("CCASN-1","00786",' ',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(22, 50, 0), raw_value: "H200214225000".to_string() })),
            ("UMOY1","237",'(',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(23, 0, 0), raw_value: "H200214230000".to_string() })),
            ("UMOY2","238",'*',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(23, 0, 0), raw_value: "H200214230000".to_string() })),
            ("UMOY3","236",')',Some(TeleinfoDate { season: 'H', date: Local.ymd(2020, 2, 14).and_hms(23, 0, 0), raw_value: "H200214230000".to_string() })),
            ("STGE","463A0800",'K',None),
            ("DPM1","00",'\\',Some(TeleinfoDate { season: ' ', date: Local.ymd(2020, 2, 14).and_hms(6, 0, 0), raw_value: " 200214060000".to_string() })),
            ("FPM1","00",'_',Some(TeleinfoDate { season: ' ', date: Local.ymd(2020, 2, 15).and_hms(6, 0, 0), raw_value: " 200215060000".to_string() })),
            ("MSG1","PAS DE          MESSAGE         ",'<',None),
            ("PRM","07361794479930",'F',None),
            ("RELAIS","001",'C',None),
            ("NTARF","03",'P',None),
            ("NJOURF","00",'&',None),
            ("NJOURF+1","00",'B',None),
            ("PJOURF+1","00004001 06004002 22004001 NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE",'.',None)];
        let expect = TeleinfoMessage {
            values: parsed_vector_to_values(expect_values),
            mode: TeleinfoMode::Standard,
            valid: true,
        };
        let expect_values_inc = vec![
            ("ADSC", "041776199277", 'I', None),
            ("VTIC", "02", 'J', None),
            (
                "DATE",
                "",
                '=',
                Some(TeleinfoDate {
                    season: 'H',
                    date: Local.ymd(2020, 2, 14).and_hms(23, 8, 6),
                    raw_value: "H200214230806".to_string(),
                }),
            ),
            ("NGTF", "     TEMPO      ", 'F', None),
            ("LTARF", "   HC  BLANC    ", '6', None),
            ("EAST", "021849107", '/', None),
            ("EASF01", "004855593", 'I', None),
            ("EASF02", "014090959", 'H', None),
            ("EASF03", "000487132", '=', None),
            ("EASF04", "001481464", 'A', None),
            ("EASF05", "000227596", 'E', None),
            ("EASF06", "000706363", '@', None),
            ("EASF07", "000000000", '(', None),
            ("EASF08", "000000000", ')', None),
            ("EASF09", "000000000", '*', None),
            ("EASF10", "000000000", '"', None),
        ];
        let expect_inc = TeleinfoMessage {
            values: parsed_vector_to_values(expect_values_inc),
            mode: TeleinfoMode::Standard,
            valid: false,
        };
        let (remain, result) = get_message(&mut stream, "".to_string()).unwrap();
        assert_eq!( (remain.clone(),result) ,
 ("\u{2}\nADSC\t041776199277\tI\r\nVTIC\t02\tJ\r\nDATE\tH200214230806\t\t=\r\nNGTF\t     TEMPO      \tF\r\nLTARF\t   HC  BLANC    \t6\r\nEAST\t021849107\t/\r\nEASF01\t004855593\tI\r\nEASF02\t014".to_string(),expect));
        let (remain2, result2) = get_message(&mut stream, remain).unwrap();
        assert_eq!( (remain2,result2) ,
 ("\u{2}\nADSC\t041776199277\tI\r\nVTIC\t02\tJ\r\nDATE\tH200214230807\t\t>\r\nNGTF\t     TEMPO      \tF\r\nLTARF\t   H".to_string(),expect_inc));
    }
}
