use nom::{
    IResult,
    bytes::streaming::{tag},
    character::streaming::{alphanumeric1,digit1,alpha1},
    character::complete::multispace0,
    number::streaming::double,
    sequence::{separated_pair,pair},
    combinator::{peek,recognize,opt},
    branch::permutation,
    multi::separated_list,
    error,
};
use chrono::{NaiveDateTime,NaiveDate};
use std::str;

use crate::common::*;

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
        let (input, interval_length) = section_of_exact_length(digit1, 2)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, next_scheduled_read_date) = match date_8(input) {
            Ok(d) => (d.0,Some(d.1)),
            Err(nom::Err::Incomplete(_)) => {
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
    #[derive(Clone,Debug,PartialEq)]
    pub struct IntervalEvent<'a> {
        pub start_interval: Input<'a>,
        pub end_interval: Input<'a>,
        pub quality_method: Input<'a>,
        pub reason_code: Input<'a>,
        pub reason_description: Option<Input<'a>>,
    }

    impl <'a>IntervalEvent<'a> {
        pub fn parse(input: Input<'a>) -> IResult<Input<'a>,IntervalEvent> {
            interval_event(input)
        }
    }

    fn interval_event<'a>(input: Input<'a>) -> IResult<Input<'a>,IntervalEvent<'a>> {
        let (input, _) = tag("400,")(input)?;
        let (input, start_interval) = section_of_max_length(digit1,4)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, end_interval) = section_of_max_length(digit1,4)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, quality_method) = section_of_max_length(alphanumeric1,3)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, reason_code) = section_of_max_length(digit1,3)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, reason_description) = optional_field(section_of_max_length(alphanumeric1,24),"\n")(input)?;

        let interval_event = IntervalEvent {
            start_interval,
            end_interval,
            quality_method,
            reason_code,
            reason_description
        };

        Ok((input,interval_event))
    }

    // B2B details record (500)
    #[derive(Clone,Debug,PartialEq)]
    pub struct B2BDetails<'a> {
        pub trans_code: Input<'a>,
        pub ret_service_order: Input<'a>,
        pub read_datetime: NaiveDateTime,
        pub index_read: Input<'a>,
    }

    impl B2BDetails<'_> {
        pub fn parse(input: Input) -> IResult<Input,B2BDetails> {
            b2b_details(input)
        }
    }

    fn b2b_details<'a>(input: Input<'a>) -> IResult<Input<'a>,B2BDetails<'a>> {
        let (input, _) = tag("500,")(input)?;
        let (input, trans_code) = section_of_exact_length(alpha1,1)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, ret_service_order) = section_of_max_length(alphanumeric1,15)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, read_datetime) = datetime_14(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, index_read) = section_of_max_length(
            move |i| recognize(permutation((digit1,opt(pair(tag("."),digit1)))))(i)
        ,15)(input)?;

        let b2b_details = B2BDetails {
            trans_code,
            ret_service_order,
            read_datetime,
            index_read
        };

        Ok((input,b2b_details))
    }

    // End of data (900)
    #[derive(Clone,Debug,PartialEq)]
    pub struct EndOfData {}

    impl EndOfData {
        pub fn parse(input: Input) -> IResult<Input,EndOfData> {
            end_of_data(input)
        }
    }

    fn end_of_data(input: Input) -> IResult<Input,EndOfData> {
        let (input, _) = tag("900")(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input,EndOfData {}))
    }
}

#[cfg(test)]
mod tests {
    use super::{record,Input,error};
    use chrono::{NaiveDate};

    #[test]
    fn header_100() {
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
    fn nmi_data_details_200() {
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
    fn interval_data_300() {
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

    #[test]
    fn interval_event_400() {
        let interval_event = record::IntervalEvent {
            start_interval: b"1",
            end_interval: b"20",
            quality_method: b"F14",
            reason_code: b"76",
            reason_description: None,
        };

        let raw = b"400,1,20,F14,76,\n";
        let res = record::IntervalEvent::parse(raw).unwrap();
        assert_eq!(res,(b"\n" as &[u8],interval_event));
    }

    #[test]
    fn b2b_details_500() {
        let interval_event = record::B2BDetails {
            trans_code: b"S",
            ret_service_order: b"RETNSRVCEORD1",
            read_datetime: NaiveDate::from_ymd(2003,12,20).and_hms(15,45,0),
            index_read: b"001123.5",
        };

        let raw = b"500,S,RETNSRVCEORD1,20031220154500,001123.5\n";
        let res = record::B2BDetails::parse(raw);
        assert_eq!(res,Ok((b"\n" as &[u8],interval_event)));
    }

    #[test]
    fn end_of_data_900() {
        let end_of_data = record::EndOfData {};

        let raw = b"900\n";
        let res = record::EndOfData::parse(raw);
        assert_eq!(res,Ok((b"" as &[u8],end_of_data)));
    }
}