use nom::{
    IResult,
    bytes::streaming::{take,tag},
    character::streaming::{alphanumeric1,digit1,alpha1},
    number::streaming::double,
    sequence::separated_pair,
    combinator::{verify,peek},
    multi::separated_list,
    error,
};
use chrono::{NaiveDateTime,NaiveDate};

use std::str;
use std::fmt;

type Input<'a> = &'a [u8];

fn section_of_max_length<'a, I: Clone, E: error::ParseError<I>, F: Copy>(
    test: F,
    length: usize
) -> impl Fn(I) -> IResult<I, Input<'a>, E>
where
    F: Fn(I) -> IResult<I, Input<'a>, E>
{
    verify(test, move |s: Input| (s.len() <= length) && (s.len() > 0))
}

fn section_of_exact_length<'a, I: Clone, E: error::ParseError<I>, F: Copy>(
    test: F,
    length: usize
) -> impl Fn(I) -> IResult<I, Input<'a>, E>
where
    F: Fn(I) -> IResult<I, Input<'a>, E>
{
    verify(test, move |s: Input| s.len() == length)
}

use nom::{InputTake,Compare,InputLength};

fn optional_field<I1, T, O, E1, F>(
    test: F,
    end_marker: T
) -> impl Fn(I1) -> IResult<I1, Option<O>, E1>
where
    I1: fmt::Debug + Clone + InputTake + Compare<T>,
    //I2: Clone + InputTake + Compare<T>,
    T: InputLength + Clone + Copy,
    E1: fmt::Debug + error::ParseError<I1>,
    //E2: error::ParseError<I2>,
    O: fmt::Debug,
    F: Fn(I1) -> IResult<I1, O, E1>
{
    move |input: I1| {
        let i = input.clone();
        match test(input) {
            Ok(d) => Ok((d.0,Some(d.1))),
            Err(nom::Err::Error(x)) => {
                match peek(tag::<T,I1,E1>(end_marker))(i) {
                    Ok((input,_)) => Ok((input,None)),
                    Err(nom::Err::Error(e)) => {
                        return Err(nom::Err::Error(e))
                    }
                    x => { println!("'{:?}'", x); panic!("This should never happen") }
                }
            },
            x => { println!("{:?}", x); panic!("This should never happen") }
        }
    }
}

fn datetime_14(input: Input) -> IResult<Input,NaiveDateTime> {
    let (input, date_time_str) = take(14usize)(input)?;

    let date_time_str = match str::from_utf8(date_time_str) {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::TakeUntil)))
    }?;
    let date_time: NaiveDateTime = match NaiveDateTime::parse_from_str(date_time_str,"%Y%m%d%H%M%S") {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::ParseTo)))
    }?;

    Ok((input,date_time))
}

fn datetime_12(input: Input) -> IResult<Input,NaiveDateTime> {
    let (input, date_time_str) = take(12usize)(input)?;

    let date_time_str = match str::from_utf8(date_time_str) {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::TakeUntil)))
    }?;
    let date_time: NaiveDateTime = match NaiveDateTime::parse_from_str(date_time_str,"%Y%m%d%H%M") {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::ParseTo)))
    }?;

    Ok((input,date_time))
}

fn date_8(input: Input) -> IResult<Input,NaiveDate> {
    let (input, date_time_str) = take(8usize)(input)?;

    let date_time_str = match str::from_utf8(date_time_str) {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::TakeUntil)))
    }?;
    let date_time: NaiveDate = match NaiveDate::parse_from_str(date_time_str,"%Y%m%d") {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::ParseTo)))
    }?;

    Ok((input,date_time))
}

pub mod record {
    use super::*;

    // Header record (100)
    #[derive(Clone,Debug,PartialEq)]
    pub struct Header<'a> {
        created: NaiveDateTime,
        from_participant: Input<'a>,
        to_participant: Input<'a>,
    }

    impl <'a>Header<'a> {
        pub fn new(created: NaiveDateTime, from_participant: Input<'a>, to_participant: Input<'a>) -> Self {
            Header {
                created,
                from_participant,
                to_participant
            }
        }

        pub fn parse(input: Input) -> IResult<Input,Header> {
            header(input)
        }
    }

    fn header(input: Input) -> IResult<Input,Header> {
        let (input, _) = tag("100,")(input)?;
        let (input, _) = tag("NEM12,")(input)?;
        let (input, created) = datetime_12(input)?;

        let (input, _) = tag(",")(input)?;

        let (input,(from_participant,to_participant)) = separated_pair(
            section_of_max_length(alphanumeric1,10),
            tag(","),
            section_of_max_length(alphanumeric1,10),
        )(input)?;

        let header = Header::new(
            created,
            from_participant,
            to_participant
        );

        Ok((input,header))
    }

    // NMI data details record (200)
    #[derive(Clone,Debug,PartialEq)]
    pub struct NMIDataDetails<'a> {
        pub nmi: Input<'a>,
        pub nmi_configuration: Input<'a>,
        pub register_id: Input<'a>,
        pub nmi_suffix: Input<'a>,
        pub mdm_data_stream_id: Input<'a>,
        pub meter_serial_number: Input<'a>,
        pub uom: Input<'a>,
        pub interval_length: usize,
        pub next_scheduled_read_date: Option<NaiveDate>,
    }

    impl <'a>NMIDataDetails<'a> {
        pub fn parse(input: Input) -> IResult<Input,NMIDataDetails> {
            nmi_data_details(input)
        }
    }

    fn nmi_data_details(input: Input) -> IResult<Input,NMIDataDetails> {
        let (input, _) = tag("200,")(input)?;
        let (input, nmi) = section_of_exact_length(alphanumeric1, 10)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, nmi_configuration) = section_of_max_length(alphanumeric1, 240)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, register_id) = section_of_max_length(alphanumeric1, 10)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, nmi_suffix) = section_of_exact_length(alphanumeric1, 2)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, mdm_data_stream_id) = section_of_exact_length(alphanumeric1, 2)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, meter_serial_number) = section_of_max_length(alphanumeric1, 12)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, uom) = section_of_max_length(alphanumeric1, 5)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, interval_length) = section_of_exact_length(digit1, 2)(input)?; // TODO: Convert to unsigned int
        let (input, _) = tag(",")(input)?;
        let (input, next_scheduled_read_date) = match date_8(input) {
            Ok(d) => (d.0,Some(d.1)),
            Err(nom::Err::Incomplete(x)) => {
                match peek(tag("\n"))(input) {
                    Ok((input,_)) => (input,None),
                    Err(nom::Err::Error((x,e))) => {
                        return Err(nom::Err::Error((x,e)))
                    }
                    x => { println!("'{:?}'", x); panic!("This should never happen") }
                }
            },
            x => { println!("{:?}", x); panic!("This should never happen") }
        };

        let nmi_data_details  = NMIDataDetails {
            nmi,
            nmi_configuration,
            register_id,
            nmi_suffix,
            mdm_data_stream_id,
            meter_serial_number,
            uom,
            interval_length: usize::from_str_radix(str::from_utf8(interval_length).unwrap(),10).unwrap(),
            next_scheduled_read_date,
        };

        Ok((input,nmi_data_details))
    }

    // Interval data record (300)
    #[derive(Clone,Debug,PartialEq)]
    pub struct IntervalData<'a> {
        pub interval_date: NaiveDate,
        pub interval_value: Vec<f64>,
        pub quality_method: Input<'a>,
        pub reason_code: Option<Input<'a>>,
        pub reason_description: Option<Input<'a>>,
        pub update_datetime: NaiveDateTime,
        pub msats_load_datetime: NaiveDateTime,
    }

    impl <'a>IntervalData<'a> {
        pub fn parse(input: Input<'a>, nmi_data_details_rec: Option<NMIDataDetails>) -> IResult<Input<'a>,IntervalData<'a>> {
            interval_data(input, nmi_data_details_rec)
        }
    }

    fn interval_data<'a>(input: Input<'a>,nmi_data_details_rec: Option<NMIDataDetails>) -> IResult<Input<'a>,IntervalData<'a>> {
        let (input, _) = tag("300,")(input)?;
        let (input, interval_date) = date_8(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, interval_value) = separated_list(tag(","),double)(input)?;

        if let Some(details) = nmi_data_details_rec {
            if (1440 / details.interval_length) != interval_value.len() {
                return Err(nom::Err::Error(error::make_error(input,error::ErrorKind::SeparatedList)))
            }
        }

        let (input, _) = tag(",")(input)?;
        let (input, quality_method) = section_of_max_length(alpha1, 3)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, reason_code) = optional_field(section_of_max_length(digit1, 3),",")(input)?;

        let (input, _) = tag(",")(input)?;
        let (input, reason_description) = optional_field(section_of_max_length(alphanumeric1, 240),",")(input)?;

        let (input, _) = tag(",")(input)?;
        let (input, update_datetime) = datetime_14(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, msats_load_datetime) = match datetime_14(input) {
            Ok(d) => d,
            Err(nom::Err::Error((x,error::ErrorKind::ParseTo))) => {
                return Err(nom::Err::Error((x,error::ErrorKind::ParseTo)))
            },
            x => { println!("{:?}", x); panic!("This should never happen") }
        };

        let interval_data = IntervalData {
            interval_date,
            interval_value,
            quality_method,
            reason_code,
            reason_description,
            update_datetime,
            msats_load_datetime
        };

        Ok((input,interval_data))
    }

    // Interval event record (400)
    // fn interval_event

    // B2B details record (500)
    // fn b2b_details

    // End of data (900)
    // fn end_of_data
}

#[cfg(test)]
mod tests {
    use super::{record,Input,error};
    use chrono::{NaiveDate};

    #[test]
    fn header() {
        let date = NaiveDate::from_ymd(2004,5,1).and_hms(11, 35, 0);
        let header = record::Header::new (
            date.clone(),
            b"MDA1",
            b"Ret1"
        );

        let raw = b"100,NEM12,200405011135,MDA1,Ret1\n";

        let res: (Input, record::Header) = match record::Header::parse(raw) {
            Ok(o) => o,
            Err(e) => { println!("{:?}",e); panic!("Failed") }
        };

        assert_eq!(res,(b"\n" as &[u8],header));

        let header = record::Header::new (
            date.clone(),
            b"0123456789",
            b"Ret1"
        );

        let raw = b"100,NEM12,200405011135,0123456789,Ret1\n";

        let res: (Input, record::Header) = match record::Header::parse(raw) {
            Ok(o) => o,
            Err(e) => { println!("{:?}",e); panic!("Failed") }
        };

        assert_eq!(res,(b"\n" as &[u8],header));

        let raw = b"100,NEM12,200405011135,12345678910,Ret1\n";

        let res: (Input, error::ErrorKind) = match record::Header::parse(raw) {
            Ok(o) => { println!("{:?}",o); panic!("Failed") },
            Err(nom::Err::Error(e)) => { e },
            Err(nom::Err::Incomplete(_)) |
            Err(nom::Err::Failure(_)) => panic!("This should never happen")
        };

        assert_eq!(res,(b"12345678910,Ret1\n" as &[u8],error::ErrorKind::Verify));
    }

    #[test]
    fn nmi_data_details() {
        let nmi_data_details = record::NMIDataDetails {
            nmi: b"VABD000163",
            nmi_configuration: b"E1Q1",
            register_id: b"1",
            nmi_suffix: b"E1",
            mdm_data_stream_id: b"N1",
            meter_serial_number: b"METSER123",
            uom: b"kWh",
            interval_length: 30usize,
            next_scheduled_read_date: None,
        };

        let raw = b"200,VABD000163,E1Q1,1,E1,N1,METSER123,kWh,30,\n";

        let res = match record::NMIDataDetails::parse(raw) {
            Ok(o) => o,
            Err(e) => { println!("{:?}",e); panic!("Failed") }
        };

        assert_eq!(res,(b"\n" as &[u8],nmi_data_details));

        let raw = b"200,VABD000163,E1Q1,1,E1,N1,METSER123,kWh,30,1234\n";

        let res = match record::NMIDataDetails::parse(raw) {
            Ok(o) => { println!("{:?}",o); panic!("Failed") },
            Err(e) => e
        };

        assert_eq!(res,nom::Err::Error(error::make_error(b"1234\n" as &[u8],error::ErrorKind::Tag)));
    }

    #[test]
    fn interval_data() {
        let interval_data = record::IntervalData {
            interval_date: NaiveDate::from_ymd(2004, 2, 1),
            interval_value: vec![1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111],
            quality_method: b"A",
            reason_code: None,
            reason_description: None,
            update_datetime: NaiveDate::from_ymd(2004, 2, 2).and_hms(12, 0, 25),
            msats_load_datetime: NaiveDate::from_ymd(2004, 2, 2).and_hms(14, 25, 16),
        };

        let raw = b"300,20040201,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,A,,,20040202120025,20040202142516\n";

        let res = match record::IntervalData::parse(raw,None) {
            Ok(o) => o,
            Err(e) => { println!("{:?}",e); panic!("Failed") }
        };

        assert_eq!(res,(b"\n" as &[u8],interval_data));
    }
}