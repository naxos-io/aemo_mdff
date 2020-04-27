use nom::{
    IResult,
    bytes::complete::{tag},
    character::complete::{alphanumeric1,digit1,alpha1},
    number::complete::double,
    character::complete::multispace0,
    sequence::pair,
    combinator::{peek,recognize,opt,map},
    branch::{alt,permutation},
    multi::separated_list,
    error,
};

use chrono::{NaiveDateTime,NaiveDate};
use ndarray::Array;
use std::str;

use crate::common::*;

pub struct NEM12<'a> {
    header: record::Header<'a>,
    nmi_data_details: Vec<NMIDetails<'a>>
}

struct Parser<'a> {
    src: Input<'a>,
    iter: std::iter::Enumerate<str::Lines<'a>>,
    data: Option<NEM12<'a>>,
    finished: bool
}

impl <'a>Parser<'a> {

    fn new(src: Input<'a>) -> Result<Self,&str> {
        let parser = Parser {
            src,
            iter: src.lines().enumerate(),
            data: None,
            finished: false
        };

        Ok(parser)
    }

    fn parse_line(&mut self) -> Result<Option<record::Kind>,(&str,usize,&str)> {
        if let Some((line_no,line)) = self.iter.next() {
            // println!("PARSING {}: '{}'",line_no,line);
            let res = alt((
                map(record::Header::parse, |o| record::Kind::Header(o)),
                map(record::NMIDataDetails::parse, |o| record::Kind::NMIDataDetails(o)),
                map(record::IntervalData::parse, |o| record::Kind::IntervalData(o)),
                map(record::IntervalEvent::parse, |o| record::Kind::IntervalEvent(o)),
                map(record::B2BDetails::parse, |o| record::Kind::B2BDetails(o)),
                map(record::EndOfData::parse, |o| record::Kind::EndOfData(o)),
            ))(line)
                .map_err(|err| {
                    match err {
                        nom::Err::Error((sentinel,_)) => ("Error parsing line: ",line_no+1,sentinel),
                        nom::Err::Failure((sentinel,_)) => ("Failed to parse line: ",line_no+1,sentinel),
                        nom::Err::Incomplete(needed) => match needed {
                            nom::Needed::Size(sz) => ("Failed to parse line: ",sz,"Needed::Size"),
                            nom::Needed::Unknown => ("Failed to parse line: ",line_no+1,"Needed::Unknown")
                        }
                    }
                })?;

            return Ok(Some(res.1));
        } else {
            match !self.finished {
                true => {
                    self.finished = true;
                    return Ok(None);
                }
                false => return Err(("Error: parser consumed all input",0,"lines"))
            }
        };
    }
}

struct NMIDetails<'a> {
    rec: record::NMIDataDetails<'a>,
    data: Array<f64,DataDetails<'a>>
}

#[derive(Clone,Debug,PartialEq)]
enum DataDetails<'a> {
    Details(&'a record::NMIDataDetails<'a>),
    Event(&'a record::IntervalEvent<'a>)
}

pub mod file {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        const MULTIPLE_METERS_STR: &'static str = "100,NEM12,200402070911,MDA1,Ret1\n\
        200,NCDE001111,E1B1Q1E2,1,E1,N1,METSER123,Wh,15,\n\
        300,20031204,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,A,,,20031206011132,20031207011022\n\
        300,20031205,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,A,,,20031206011132,20031207011022\n\
        200,NCDE001111,E1B1Q1E2,2,B1,N1,METSER123,Wh,15,\n\
        300,20031204,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,A,,,20031206011132,20031207011022\n\
        300,20031205,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,A,,,20031206011132,20031207011022\n\
        200,NCDE001111,E1B1Q1E2,3,Q1,,METSER123,VArh,15,\n\
        300,20031204,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,A,,,20031206011155,\n\
        300,20031205,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,A,,,20031206011155,\n\
        200,NCDE001111,E1B1Q1E2,4,E2,N2,METSER456,Wh,15,\n\
        300,20031204,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,A,,,20031206011140,20031207011022\n\
        300,20031205,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,100,A,,,20031206011140,20031207011022\n\
        200,NDDD001888,B1K2,1,B1,N1,METSER991,Wh,15,\n\
        300,20031204,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,A,,,20031206011145,20031207011022\n\
        300,20031205,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,A,,,20031206011145,20031207011022\n\
        200,NDDD001888,B1K2,2,K2,,METSER992,VArh,15,\n\
        300,20031204,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,A,,,20031206011155,\n\
        300,20031205,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,50,A,,,20031206011155,\n\
        900";

        #[test]
        fn multiple_meters() {

            let mut parser = Parser::new(MULTIPLE_METERS_STR).unwrap();

            for _ in 1..20 {
                parser.parse_line().unwrap();
            }

            assert_eq!(parser.parse_line(),Ok(Some(record::Kind::EndOfData(record::EndOfData{}))));
            assert_eq!(parser.parse_line(),Ok(None));
            assert_eq!(parser.parse_line(),Err(("Error: parser consumed all input",0,"lines")));
        }
    }
}

pub mod record {
    use super::*;

    #[derive(Clone,Debug,PartialEq)]
    pub enum Kind<'a> {
        Header(Header<'a>),
        NMIDataDetails(NMIDataDetails<'a>),
        IntervalData(IntervalData<'a>),
        IntervalEvent(IntervalEvent<'a>),
        B2BDetails(B2BDetails<'a>),
        EndOfData(EndOfData)
    }

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
        let (input, from_participant) = section_of_max_length(alphanumeric1,10)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, to_participant) = section_of_max_length(alphanumeric1,10)(input)?;

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
        pub mdm_data_stream_id: Option<Input<'a>>,
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
        let (input, mdm_data_stream_id) = optional_field(section_of_exact_length(alphanumeric1, 2),",")(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, meter_serial_number) = section_of_max_length(alphanumeric1, 12)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, uom) = section_of_max_length(alphanumeric1, 5)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, interval_length) = section_of_exact_length(digit1, 2)(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, next_scheduled_read_date) = match date_8(input){
            Ok(d) => Ok((d.0,Some(d.1))),
            Err(nom::Err::Error(_)) => {
                match peek(alt((eof,tag("\n"))))(input) { // TODO: Add alt(eof,tag) to optional_field
                    Ok((input,_)) => Ok((input,None)),
                    Err(nom::Err::Error(e)) => {
                        return Err(nom::Err::Error(e))
                    }
                    x => { println!("'{:?}'", x); panic!("This should never happen") }
                }
            },
            x => { println!("{:?}", x); panic!("This should never happen") }
        }?;

        let nmi_data_details  = NMIDataDetails {
            nmi,
            nmi_configuration,
            register_id,
            nmi_suffix,
            mdm_data_stream_id,
            meter_serial_number,
            uom,
            interval_length: usize::from_str_radix(interval_length,10).unwrap(),
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
        pub msats_load_datetime: Option<NaiveDateTime>,
    }

    impl <'a>IntervalData<'a> {
        pub fn parse(input: Input<'a>) -> IResult<Input<'a>,IntervalData<'a>> {
            interval_data(input)
        }
    }

    fn interval_data<'a>(input: Input<'a>) -> IResult<Input<'a>,IntervalData<'a>> {
            let (input, _) = tag("300,")(input)?;
        let (input, interval_date) = date_8(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, interval_value) = separated_list(tag(","),double)(input)?;

        // if let Some(details) = nmi_data_details_rec {
        //     if (1440 / details.interval_length) != interval_value.len() {
        //         return Err(nom::Err::Error(error::make_error(input,error::ErrorKind::SeparatedList)))
        //     }
        // }

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
            Ok((input,d)) => (input,Some(d)),
            Err(nom::Err::Error((input,error::ErrorKind::Eof))) => {
                (input,None)
            },
            Err(nom::Err::Error((input,error::ErrorKind::ParseTo))) => {
                return Err(nom::Err::Error((input,error::ErrorKind::ParseTo)))
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

    #[cfg(test)]
    mod tests {
        use super::{record,Input,error};
        use chrono::{NaiveDate};
    
        #[test]
        fn header_100() {
            let date = NaiveDate::from_ymd(2004,5,1).and_hms(11, 35, 0);
            let header = record::Header::new (
                date.clone(),
                "MDA1",
                "Ret1"
            );
    
            let raw = "100,NEM12,200405011135,MDA1,Ret1\n";
    
            let res: (Input, record::Header) = match record::Header::parse(raw) {
                Ok(o) => o,
                Err(e) => { println!("{:?}",e); panic!("Failed") }
            };
    
            assert_eq!(res,("\n",header));
    
            let header = record::Header::new (
                date.clone(),
                "0123456789",
                "Ret1"
            );
    
            let raw = "100,NEM12,200405011135,0123456789,Ret1\n";
    
            let res: (Input, record::Header) = match record::Header::parse(raw) {
                Ok(o) => o,
                Err(e) => { println!("{:?}",e); panic!("Failed") }
            };
    
            assert_eq!(res,("\n",header));
    
            let raw = "100,NEM12,200405011135,12345678910,Ret1\n";
    
            let res: (Input, error::ErrorKind) = match record::Header::parse(raw) {
                Ok(o) => { println!("{:?}",o); panic!("Failed") },
                Err(nom::Err::Error(e)) => { e },
                Err(nom::Err::Incomplete(_)) |
                Err(nom::Err::Failure(_)) => panic!("This should never happen")
            };
    
            assert_eq!(res,("12345678910,Ret1\n",error::ErrorKind::Verify));
        }
    
        #[test]
        fn nmi_data_details_200() {
            let nmi_data_details = record::NMIDataDetails {
                nmi: "VABD000163",
                nmi_configuration: "E1Q1",
                register_id: "1",
                nmi_suffix: "E1",
                mdm_data_stream_id: Some("N1"),
                meter_serial_number: "METSER123",
                uom: "KWH",
                interval_length: 30usize,
                next_scheduled_read_date: None,
            };
    
            let raw = "200,VABD000163,E1Q1,1,E1,N1,METSER123,KWH,30,\n";
    
            let res = match record::NMIDataDetails::parse(raw) {
                Ok(o) => o,
                Err(e) => { println!("{:?}",e); panic!("Failed") }
            };
    
            assert_eq!(res,("\n",nmi_data_details));
    
            let raw = "200,VABD000163,E1Q1,1,E1,N1,METSER123,kWh,30,1234\n";
    
            let res = match record::NMIDataDetails::parse(raw) {
                Ok(o) => { println!("{:?}",o); panic!("Failed") },
                Err(e) => e
            };
    
            assert_eq!(res,nom::Err::Error(error::make_error("1234\n",error::ErrorKind::Tag)));
        }
    
        #[test]
        fn interval_data_300() {
            let interval_data = record::IntervalData {
                interval_date: NaiveDate::from_ymd(2004, 2, 1),
                interval_value: vec![1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111],
                quality_method: "A",
                reason_code: None,
                reason_description: None,
                update_datetime: NaiveDate::from_ymd(2004, 2, 2).and_hms(12, 0, 25),
                msats_load_datetime: Some(NaiveDate::from_ymd(2004, 2, 2).and_hms(14, 25, 16)),
            };
    
            let raw = "300,20040201,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,A,,,20040202120025,20040202142516\n";
    
            let res = match record::IntervalData::parse(raw) {
                Ok(o) => o,
                Err(e) => { println!("{:?}",e); panic!("Failed") }
            };
    
            assert_eq!(res,("\n",interval_data));
        }
    
        #[test]
        fn interval_event_400() {
            let interval_event = record::IntervalEvent {
                start_interval: "1",
                end_interval: "20",
                quality_method: "F14",
                reason_code: "76",
                reason_description: None,
            };
    
            let raw = "400,1,20,F14,76,\n";
            let res = record::IntervalEvent::parse(raw).unwrap();
            assert_eq!(res,("\n",interval_event));
        }
    
        #[test]
        fn b2b_details_500() {
            let interval_event = record::B2BDetails {
                trans_code: "S",
                ret_service_order: "RETNSRVCEORD1",
                read_datetime: NaiveDate::from_ymd(2003,12,20).and_hms(15,45,0),
                index_read: "001123.5",
            };
    
            let raw = "500,S,RETNSRVCEORD1,20031220154500,001123.5\n";
            let res = record::B2BDetails::parse(raw);
            assert_eq!(res,Ok(("\n",interval_event)));
        }
    
        #[test]
        fn end_of_data_900() {
            let end_of_data = record::EndOfData {};
    
            let raw = "900\n";
            let res = record::EndOfData::parse(raw);
            assert_eq!(res,Ok(("",end_of_data)));
        }
    }
}