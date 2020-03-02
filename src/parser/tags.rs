use nom::{branch::alt, bytes::complete::tag, IResult};

fn parser_tag_legacy_1(input: &str) -> IResult<&str, &str> {
    alt((
        tag("ADCO"),
        tag("OPTARIF"),
        tag("ISOUSC"),
        tag("PEJP"),
        tag("PTEC"),
        tag("DEMAIN"),
        tag("IINST"),
        tag("IINST1"),
        tag("IINST2"),
        tag("IINST3"),
        tag("ADPS"),
        tag("IMAX"),
        tag("IMAX1"),
        tag("IMAX2"),
        tag("IMAX3"),
        tag("PAPP"),
        tag("PMAX"),
        tag("HHPHC"),
        tag("MOTDETAT"),
        tag("PPOT"),
    ))(input)
}

fn parser_tag_legacy_2(input: &str) -> IResult<&str, &str> {
    alt((
        tag("BASE"),
        tag("HCHC"),
        tag("HCHP"),
        tag("EJPHN"),
        tag("EJPHPM"),
        tag("BBRHCJB"),
        tag("BBRHPJB"),
        tag("BBRHCJW"),
        tag("BBRHPJW"),
        tag("BBRHCJR"),
        tag("BBRHPJR"),
    ))(input)
}

fn parser_tag_legacy_3(input: &str) -> IResult<&str, &str> {
    alt((tag("ADIR1"), tag("ADIR2"), tag("ADIR3")))(input)
}

pub fn parser_tag_legacy(input: &str) -> IResult<&str, &str> {
    alt((
        parser_tag_legacy_1,
        parser_tag_legacy_2,
        parser_tag_legacy_3,
    ))(input)
}

fn parser_tag_standard_1(input: &str) -> IResult<&str, &str> {
    alt((
        tag("ADSC"),
        tag("VTIC"),
        tag("NGTF"),
        tag("LTARF"),
        tag("EAST"),
        tag("EASF01"),
        tag("EASF02"),
        tag("EASF03"),
        tag("EASF04"),
        tag("EASF05"),
        tag("EASF06"),
        tag("EASF07"),
        tag("EASF08"),
        tag("EASF09"),
        tag("EASF10"),
        tag("EASD01"),
        tag("EASD02"),
        tag("EASD03"),
        tag("EASD04"),
        tag("EAIT"),
    ))(input)
}

fn parser_tag_standard_2(input: &str) -> IResult<&str, &str> {
    alt((
        tag("ADSC"),
        tag("ERQ1"),
        tag("ERQ2"),
        tag("ERQ3"),
        tag("ERQ4"),
        tag("IRMS1"),
        tag("IRMS2"),
        tag("IRMS3"),
        tag("URMS1"),
        tag("URMS2"),
        tag("URMS3"),
        tag("PREF"),
        tag("PCOUP"),
        tag("SINSTS1"),
        tag("SINSTS2"),
        tag("SINSTS3"),
        tag("SINSTS"),
        tag("SINSTI"),
        tag("STGE"),
        tag("MSG1"),
        tag("MSG2"),
    ))(input)
}

fn parser_tag_standard_3(input: &str) -> IResult<&str, &str> {
    alt((
        tag("ADSC"),
        tag("PRM"),
        tag("RELAIS"),
        tag("NTARF"),
        tag("NJOURF+1"),
        tag("NJOURF"),
        tag("PJOURF+1"),
        tag("PPOINTE"),
    ))(input)
}

pub fn parser_tag_standard(input: &str) -> IResult<&str, &str> {
    alt((
        parser_tag_standard_1,
        parser_tag_standard_2,
        parser_tag_standard_3,
    ))(input)
}

fn parser_tag_standard_horodate_1(input: &str) -> IResult<&str, &str> {
    alt((
        tag("DATE"),
        tag("SMAXSN1-1"),
        tag("SMAXSN2-1"),
        tag("SMAXSN3-1"),
        tag("SMAXSN-1"),
        tag("SMAXSN1"),
        tag("SMAXSN2"),
        tag("SMAXSN3"),
        tag("SMAXSN"),
        tag("SMAXIN-1"),
        tag("SMAXIN"),
        tag("CCASN-1"),
        tag("CCASN"),
        tag("CCAIN-1"),
        tag("CCAIN"),
        tag("UMOY1"),
        tag("UMOY2"),
        tag("UMOY3"),
        tag("DPM1"),
        tag("FPM1"),
        tag("DPM2"),
    ))(input)
}

fn parser_tag_standard_horodate_2(input: &str) -> IResult<&str, &str> {
    alt((tag("FPM2"), tag("DPM3"), tag("FPM3")))(input)
}

pub fn parser_tag_standard_horodate(input: &str) -> IResult<&str, &str> {
    alt((
        parser_tag_standard_horodate_1,
        parser_tag_standard_horodate_2,
    ))(input)
}
