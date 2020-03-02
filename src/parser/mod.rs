use nom::{
    branch::alt, bytes::complete::tag, bytes::complete::tag_no_case, bytes::complete::take,
    bytes::complete::take_until, bytes::streaming::take_until as stream_take_until,
    character::complete::anychar, character::complete::char, character::complete::digit1,
    combinator::recognize, combinator::verify, multi::many1, sequence::delimited, sequence::tuple,
    IResult,
};

use crate::parser::tags::*;
use crate::{TeleinfoDate, TeleinfoMode, TeleinfoTuple};
use chrono::{Local, TimeZone};

mod tags;

fn get_beginning(input: &str) -> IResult<&str, &str> {
    recognize(tuple((stream_take_until("\u{02}"), tag("\u{02}"))))(input)
}

pub fn get_message(input: &str) -> IResult<&str, &str> {
    delimited(get_beginning, stream_take_until("\u{03}"), tag("\u{03}"))(input)
}

fn separator(mode: TeleinfoMode) -> char {
    match mode {
        TeleinfoMode::Standard => '\t',
        TeleinfoMode::Legacy => ' ',
    }
}

fn parser_value_legacy(input: &str) -> IResult<&str, &str> {
    parser_value_helper(input, TeleinfoMode::Legacy)
}

fn parser_value_standard(input: &str) -> IResult<&str, &str> {
    parser_value_helper(input, TeleinfoMode::Standard)
}

fn parser_value_helper(input: &str, mode: TeleinfoMode) -> IResult<&str, &str> {
    take_until(&(separator(mode).to_string()) as &str)(input)
}

fn parser_dataset_legacy(input: &str) -> IResult<&str, TeleinfoTuple> {
    let mode = TeleinfoMode::Legacy;
    let (input, (_, tag, _, data, _, checksum, _)) = tuple((
        char('\u{0a}'),
        parser_tag_legacy,
        char(separator(mode)),
        parser_value_legacy,
        char(separator(mode)),
        anychar,
        char('\u{0d}'),
    ))(input)?;
    Ok((input, (tag, data, checksum, None)))
}

fn parser_dataset_standard(input: &str) -> IResult<&str, TeleinfoTuple> {
    alt((
        parser_dataset_standard_nohd,
        parser_dataset_standard_horodate,
    ))(input)
}
fn parser_dataset_standard_nohd(input: &str) -> IResult<&str, TeleinfoTuple> {
    let mode = TeleinfoMode::Standard;
    let (input, (_, tag, _, data, _, checksum, _)) = tuple((
        char('\u{0a}'),
        parser_tag_standard,
        char(separator(mode)),
        parser_value_standard,
        char(separator(mode)),
        anychar,
        char('\u{0d}'),
    ))(input)?;
    Ok((input, (tag, data, checksum, None)))
}

fn parser_dataset_standard_horodate(input: &str) -> IResult<&str, TeleinfoTuple> {
    let mode = TeleinfoMode::Standard;
    let (input, (_, tag, _, date, _, data, _, checksum, _)) = tuple((
        char('\u{0a}'),
        parser_tag_standard_horodate,
        char(separator(mode)),
        parser_horodate,
        char(separator(mode)),
        parser_value_standard,
        char(separator(mode)),
        anychar,
        char('\u{0d}'),
    ))(input)?;
    Ok((input, (tag, data, checksum, Some(date))))
}

fn parser_horodate_season(input: &str) -> IResult<&str, &str> {
    alt((tag_no_case("h"), tag_no_case("e"), tag(" ")))(input)
}

fn parser_date_verifier(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

fn parser_horodate_date(input: &str) -> IResult<&str, &str> {
    verify(take(12usize), |s: &str| {
        parser_date_verifier(s).unwrap() == ("" as &str, s)
    })(input)
}

fn parser_horodate(input: &str) -> IResult<&str, TeleinfoDate> {
    match tuple((parser_horodate_season, parser_horodate_date))(input) {
        Err(e) => Err(e),
        Ok((r, (season, date))) => {
            let raw_value = format!("{}{}", season, date);
            Ok((
                r,
                TeleinfoDate {
                    season: season.chars().next().unwrap(),
                    date: Local.datetime_from_str(date, "%y%m%d%H%M%S").unwrap(),
                    raw_value,
                },
            ))
        }
    }
}

pub fn parser_message(input: &str) -> IResult<&str, (Vec<TeleinfoTuple>, TeleinfoMode)> {
    alt((parser_message_legacy, parser_message_standard))(input)
}

pub fn parser_message_legacy(input: &str) -> IResult<&str, (Vec<TeleinfoTuple>, TeleinfoMode)> {
    match many1(parser_dataset_legacy)(input) {
        Ok((r, v)) => Ok((r, (v, TeleinfoMode::Legacy))),
        Err(e) => Err(e),
    }
}

pub fn parser_message_standard(input: &str) -> IResult<&str, (Vec<TeleinfoTuple>, TeleinfoMode)> {
    match many1(parser_dataset_standard)(input) {
        Ok((r, v)) => Ok((r, (v, TeleinfoMode::Standard))),
        Err(e) => Err(e),
    }
}

pub fn validate_message(mode: TeleinfoMode, message: Vec<TeleinfoTuple>) -> bool {
    message.clone().iter().map(|m| validate(mode, m)).all(|x| x)
}

fn validate(mode: TeleinfoMode, values: &TeleinfoTuple) -> bool {
    let (tag, value, cs, hd) = values.clone();
    let include_sep = if let TeleinfoMode::Legacy = mode {
        "".to_string()
    } else {
        separator(mode).to_string()
    };
    match hd {
        None => {
            calculate_checksum(&format!(
                "{}{}{}{}",
                tag,
                separator(mode),
                value,
                include_sep
            )) == cs
        }
        Some(date) => {
            calculate_checksum(&format!(
                "{}{}{}{}{}{}",
                tag,
                separator(mode),
                date.raw_value,
                separator(mode),
                value,
                separator(mode)
            )) == cs
        }
    }
}

fn calculate_checksum(input: &str) -> char {
    ((input
        .to_string()
        .chars()
        .fold(0 as u32, |acc, c| acc + (c as u32))
        & 0x3F) as u8
        + 0x20) as char
}

#[cfg(test)]
mod tests {
    use crate::parser::get_message;
    use crate::parser::parser_dataset_legacy;
    use crate::parser::parser_dataset_standard;
    use crate::parser::parser_horodate;
    use crate::parser::parser_message;
    use crate::parser::parser_tag_standard;
    use crate::parser::validate;
    use crate::{TeleinfoDate, TeleinfoMode};
    use chrono::{Local, TimeZone};
    #[test]
    fn test_line() {
        let line_1 = "\u{0a}BBRHCJB 001478389 E\u{0d}";
        assert_eq!(
            parser_dataset_legacy(line_1),
            Ok(("", ("BBRHCJB", "001478389", 'E', None)))
        );
        let line_std_hd = "\u{0a}SMAXSN3-1\tH200213085118\t03191\tK\u{0d}";
        assert_eq!(
            parser_dataset_standard(line_std_hd),
            Ok((
                "",
                (
                    "SMAXSN3-1",
                    "03191",
                    'K',
                    Some(TeleinfoDate {
                        season: 'H',
                        date: Local.ymd(2020, 2, 13).and_hms(8, 51, 18),
                        raw_value: "H200213085118".to_string()
                    })
                )
            ))
        );
        let line_std_nohd = "\u{0a}EASF06\t000706363\t@\u{0d}";
        assert_eq!(
            parser_dataset_standard(line_std_nohd),
            Ok(("", ("EASF06", "000706363", '@', None)))
        );
    }
    #[test]
    fn test_parser_message() {
        let data = String::from_utf8_lossy(include_bytes!("../../assets/message.txt"));
        let expect = vec![
            ("ADCO", "031961098836", 'M', None),
            ("OPTARIF", "BBR(", 'S', None),
            ("ISOUSC", "45", '?', None),
            ("BBRHCJB", "001478389", 'E', None),
            ("BBRHPJB", "001012295", '>', None),
            ("BBRHCJW", "000134553", 'G', None),
            ("BBRHPJW", "000213701", 'M', None),
            ("BBRHCJR", "000025098", 'E', None),
            ("BBRHPJR", "000006010", 'A', None),
            ("PTEC", "HPJB", 'P', None),
            ("DEMAIN", "BLEU", 'V', None),
            ("IINST", "001", 'X', None),
            ("IMAX", "060", 'E', None),
            ("PAPP", "00120", '$', None),
            ("HHPHC", "A", ',', None),
            ("MOTDETAT", "000000", 'B', None),
        ];
        assert_eq!(
            parser_message(&data),
            Ok(("\n", (expect, TeleinfoMode::Legacy)))
        );
    }
    #[test]
    fn test_parser_message_standard() {
        let data = String::from_utf8_lossy(include_bytes!("../../assets/message_standard.txt"));
        let expect = vec![
             ("ADSC", "041776199277", 'I', None),
             ("VTIC", "02", 'J', None),
             ("DATE", "", ';', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(23,08,04), raw_value: "H200214230804".to_string() })),
             ("NGTF", "     TEMPO      ", 'F', None),
             ("LTARF", "   HC  BLANC    ", '6', None),
             ("EAST", "021849106", '.', None),
             ("EASF01", "004855593", 'I', None),
             ("EASF02", "014090959", 'H', None),
             ("EASF03", "000487131", '<', None),
             ("EASF04", "001481464", 'A', None),
             ("EASF05", "000227596", 'E', None),
             ("EASF06", "000706363", '@', None),
             ("EASF07", "000000000", '(', None),
             ("EASF08", "000000000", ')', None),
             ("EASF09", "000000000", '*', None),
             ("EASF10", "000000000", '\"', None),
             ("EASD01", "021849106", '?', None),
             ("EASD02", "000000000", '!', None),
             ("EASD03", "000000000", '\"', None),
             ("EASD04", "000000000", '#', None),
             ("IRMS1", "003", '1', None),
             ("IRMS2", "006", '5', None),
             ("IRMS3", "003", '3', None),
             ("URMS1", "237", 'F', None),
             ("URMS2", "238", 'H', None),
             ("URMS3", "235", 'F', None),
             ("PREF", "30", 'B', None),
             ("PCOUP", "30", '\\', None),
             ("SINSTS", "02700", 'O', None),
             ("SINSTS1", "00664", 'G', None),
             ("SINSTS2", "01373", 'F', None),
             ("SINSTS3", "00664", 'I', None),
             ("SMAXSN", "10802", '7', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(17,51,35), raw_value: "H200214175135".to_string() })),
             ("SMAXSN1", "03411", '&', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(17,51,35), raw_value: "H200214175135".to_string() })),
             ("SMAXSN2", "03899", ';', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(17,51,35), raw_value: "H200214175135".to_string() })),
             ("SMAXSN3", "03512", '*', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(17,51,35), raw_value: "H200214175135".to_string() })),
             ("SMAXSN-1", "09562", ' ', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,13).and_hms(08,51,18), raw_value: "H200213085118".to_string() })),
             ("SMAXSN1-1", "03129", 'J', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,13).and_hms(08,51,18), raw_value: "H200213085118".to_string() })),
             ("SMAXSN2-1", "03366", '@', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,13).and_hms(10,11,42), raw_value: "H200213101142".to_string() })),
             ("SMAXSN3-1", "03191", 'K', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,13).and_hms(08,51,18), raw_value: "H200213085118".to_string() })),
             ("CCASN", "01650", '5', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(23,00,00), raw_value: "H200214230000".to_string() })),
             ("CCASN-1", "00786", ' ', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(22,50,00), raw_value: "H200214225000".to_string() })),
             ("UMOY1", "237", '(', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(23,00,00), raw_value: "H200214230000".to_string() })),
             ("UMOY2", "238", '*', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(23,00,00), raw_value: "H200214230000".to_string() })),
             ("UMOY3", "236", ')', Some(TeleinfoDate { season: 'H', date: Local.ymd(2020,02,14).and_hms(23,00,00), raw_value: "H200214230000".to_string() })),
             ("STGE", "463A0800", 'K', None),
             ("DPM1", "00", '\\', Some(TeleinfoDate { season: ' ', date: Local.ymd(2020,02,14).and_hms(06,00,00), raw_value: " 200214060000".to_string() })),
             ("FPM1", "00", '_', Some(TeleinfoDate { season: ' ', date: Local.ymd(2020,02,15).and_hms(06,00,00), raw_value: " 200215060000".to_string() })),
             ("MSG1", "PAS DE          MESSAGE         ", '<', None),
             ("PRM", "07361794479930", 'F', None),
             ("RELAIS", "001", 'C', None),
             ("NTARF", "03", 'P', None),
             ("NJOURF", "00", '&', None),
             ("NJOURF+1", "00", 'B', None),
             ("PJOURF+1", "00004001 06004002 22004001 NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE", '.', None)
        ];
        assert_eq!(
            parser_message(&data),
            Ok(("\n", (expect, TeleinfoMode::Standard)))
        );
    }
    #[test]
    fn test_cs() {
        assert_eq!(
            validate(TeleinfoMode::Legacy, &("BBRHCJB", "001478389", 'E', None)),
            true
        );
        assert_eq!(
            validate(TeleinfoMode::Legacy, &("BBRHCJB", "001478389", 'F', None)),
            false
        );
        assert_eq!(
            validate(TeleinfoMode::Standard, &("EASF01", "004855593", 'I', None),),
            true
        );
        assert_eq!(
            validate(TeleinfoMode::Standard, &("EASF01", "004855593", 'J', None),),
            false
        );
    }
    #[test]
    fn test_standard_dataset() {
        assert_eq!(
            parser_tag_standard("SINSTS1\t00664\tG\r"),
            Ok(("\t00664\tG\r", "SINSTS1"))
        );
        assert_eq!(
            parser_dataset_standard("\nSINSTS1\t00664\tG\r"),
            Ok(("", ("SINSTS1", "00664", 'G', None)))
        )
    }
    #[test]
    fn test_horodate() {
        let expected = TeleinfoDate {
            season: 'H',
            date: Local.ymd(2008, 12, 25).and_hms(22, 35, 18),
            raw_value: "H081225223518".to_string(),
        };
        let expected2 = expected.clone();
        assert_eq!(parser_horodate("H081225223518"), Ok(("", expected)));
        assert_ne!(parser_horodate("D081225223518"), Ok(("", expected2)));
    }
    #[test]
    fn test_get_message() {
        let data =
            String::from_utf8_lossy(include_bytes!("../../assets/stream_legacy_complete.txt"));
        let data_standard =
            String::from_utf8_lossy(include_bytes!("../../assets/stream_standard_complete.txt"));
        let expect = vec![
            ("ADCO", "031961098836", 'M', None),
            ("OPTARIF", "BBR(", 'S', None),
            ("ISOUSC", "45", '?', None),
            ("BBRHCJB", "001478389", 'E', None),
            ("BBRHPJB", "001012295", '>', None),
            ("BBRHCJW", "000134553", 'G', None),
            ("BBRHPJW", "000213701", 'M', None),
            ("BBRHCJR", "000025098", 'E', None),
            ("BBRHPJR", "000006010", 'A', None),
            ("PTEC", "HPJB", 'P', None),
            ("DEMAIN", "BLEU", 'V', None),
            ("IINST", "001", 'X', None),
            ("IMAX", "060", 'E', None),
            ("PAPP", "00120", '$', None),
            ("HHPHC", "A", ',', None),
            ("MOTDETAT", "000000", 'B', None),
        ];
        let expect_standard = vec![
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
        let message = get_message(&data);
        let message_standard = get_message(&data_standard);
        assert_eq!(message,
            Ok(("\u{2}\nADCO 031961098836 M\r\nOPTARIF BBR( S\r\n", "\nADCO 031961098836 M\r\nOPTARIF BBR( S\r\nISOUSC 45 ?\r\nBBRHCJB 001478389 E\r\nBBRHPJB 001012295 >\r\nBBRHCJW 000134553 G\r\nBBRHPJW 000213701 M\r\nBBRHCJR 000025098 E\r\nBBRHPJR 000006010 A\r\nPTEC HPJB P\r\nDEMAIN BLEU V\r\nIINST 001 X\r\nIMAX 060 E\r\nPAPP 00120 $\r\nHHPHC A ,\r\nMOTDETAT 000000 B\r"))
        );
        assert_eq!(message_standard,
            Ok(("\u{2}\nADSC\t041776199277\tI\r\nVTIC\t02\tJ\r\nDATE\tH200214230806\t\t=\r\nNGTF\t     TEMPO      \tF\r\nLTARF\t   HC  BLANC    \t6\r\nEAST\t 21849107\t/\r\nEASF01\t004855593\tI\r\nEASF02\t014090959\tH\r\nEASF03\t000487132\t=\r\nEASF04\t001481464\tA\r\nEASF05\t000227596\tE\r\nEASF06\t000706363\t@\r\nEASF07\t000000000\t(\r\nEASF08\t000000000\t)\r\nEASF09\t000000000\t*\r\nEASF10\t000000000\t\"\r\nEASD01\t021849107\t@\u{c}\nEASD02\t000000000\t!\r\n", "\nADSC\t041776199277\tI\r\nVTIC\t02\tJ\r\nDATE\tH200214230804\t\t;\r\nNGTF\t     TEMPO      \tF\r\nLTARF\t   HC  BLANC    \t6\r\nEAST\t021849106\t.\r\nEASF01\t004855593\tI\r\nEASF02\t014090959\tH\r\nEASF03\t000487131\t<\r\nEASF04\t001481464\tA\r\nEASF05\t000227596\tE\r\nEASF06\t000706363\t@\r\nEASF07\t000000000\t(\r\nEASF08\t000000000\t)\r\nEASF09\t000000000\t*\r\nEASF10\t000000000\t\"\r\nEASD01\t021849106\t?\r\nEASD02\t000000000\t!\r\nEASD03\t000000000\t\"\r\nEASD04\t000000000\t#\r\nIRMS1\t003\t1\r\nIRMS2\t006\t5\r\nIRMS3\t003\t3\r\nURMS1\t237\tF\r\nURMS2\t238\tH\r\nURMS3\t235\tF\r\nPREF\t30\tB\r\nPCOUP\t30\t\\\r\nSINSTS\t02700\tO\r\nSINSTS1\t00664\tG\r\nSINSTS2\t01373\tF\r\nSINSTS3\t00664\tI\r\nSMAXSN\tH200214175135\t10802\t7\r\nSMAXSN1\tH200214175135\t03411\t&\r\nSMAXSN2\tH200214175135\t03899\t;\r\nSMAXSN3\tH200214175135\t03512\t*\r\nSMAXSN-1\tH200213085118\t09562\t \r\nSMAXSN1-1\tH200213085118\t03129\tJ\r\nSMAXSN2-1\tH200213101142\t03366\t@\r\nSMAXSN3-1\tH200213085118\t03191\tK\r\nCCASN\tH200214230000\t01650\t5\r\nCCASN-1\tH200214225000\t00786\t \r\nUMOY1\tH200214230000\t237\t(\r\nUMOY2\tH200214230000\t238\t*\r\nUMOY3\tH200214230000\t236\t)\r\nSTGE\t463A0800\tK\r\nDPM1\t 200214060000\t00\t\\\r\nFPM1\t 200215060000\t00\t_\r\nMSG1\tPAS DE          MESSAGE         \t<\r\nPRM\t07361794479930\tF\r\nRELAIS\t001\tC\r\nNTARF\t03\tP\r\nNJOURF\t00\t&\r\nNJOURF+1\t00\tB\r\nPJOURF+1\t00004001 06004002 22004001 NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE\t.\r")) 
        );
        match message {
            Err(_) => assert_eq!(1, 0),
            Ok((_r, m)) => assert_eq!(parser_message(m), Ok(("", (expect, TeleinfoMode::Legacy)))),
        };
        match message_standard {
            Err(_) => assert_eq!(1, 0),
            Ok((_r, m)) => assert_eq!(
                parser_message(m),
                Ok(("", (expect_standard, TeleinfoMode::Standard)))
            ),
        };
    }
    #[test]
    fn test_get_message_incomplete() {
        let data =
            String::from_utf8_lossy(include_bytes!("../../assets/stream_legacy_incomplete.txt"));
        assert_eq!(
            get_message(&data),
            Err(nom::Err::Incomplete(nom::Needed::Size(1)))
        )
    }
}
