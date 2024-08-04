use nom::{
    branch::{alt,permutation}, bytes::complete::tag, character::complete::{alpha1, alphanumeric1, digit1, multispace0}, combinator::{map, opt, peek, recognize}, error::Error, multi::separated_list1 as separated_list, number::complete::double, sequence::{pair, preceded, terminated}, Err, IResult, InputTake, Needed
};

use chrono::{NaiveDateTime,NaiveDate};
use record::{B2BDetails, EndOfData, Header, IntervalData, IntervalEvent, NMIDataDetails};
use std::{str, sync::Mutex};

use crate::common::*;

#[derive(Clone,Debug,PartialEq)]
pub struct NEM12<'a> {
    header: record::Header<'a>,
    nmi_data_details: Vec<record::NMIDataDetails<'a>>
}

fn parse_nmi_data_details<'a>(input:Input<'a>) -> IResult<Input,NMIDataDetails> {
    let (input, mut nmi_details) = terminated(NMIDataDetails::parse,rec_separator)(input)?;
    let interval_data_len = 1440 / nmi_details.interval_length;
    let (input_pre_b2b,interval_data) = opt(separated_list(rec_separator, parse_interval_data(interval_data_len)))(input)?;
    let (input,_) = rec_separator(input_pre_b2b)?;

    nmi_details.interval_data_vec = interval_data;

    if let (input,Some(b2b_details)) = opt(separated_list(rec_separator, B2BDetails::parse))(input)? {
        nmi_details.b2b_details = Some(b2b_details);
        Ok((input,nmi_details))
    } else {
        Ok((input_pre_b2b,nmi_details))
    }
    
}

fn parse_interval_data<'a>(capacity: usize) -> impl FnMut(Input<'a>) -> IResult<Input<'a>,IntervalData<'a>> {
    move |input: Input<'a>| {
        let (input_before_events, mut interval_data) = IntervalData::parse(capacity,input)?;
        let (input, _) = rec_separator(input_before_events)?;
        let (input, interval_events) = opt(separated_list(rec_separator, IntervalEvent::parse))(input)?;
        interval_data.interval_events = interval_events;

        if interval_data.interval_events.is_some() {
            Ok((input,interval_data))
        } else {
            Ok((input_before_events,interval_data))
        }
    }
}

impl <'a>NEM12<'a> {
    fn new(header: Header<'a>, nmi_data_details: Vec<NMIDataDetails<'a>>) -> Self {
        NEM12 {
            header,
            nmi_data_details
        }
    }

    fn from_str(input: Input<'a>) -> Result<NEM12<'a>,Err<Error<Input<'a>>>> {
        let strict = true;
        let (input,header) = Header::parse(input)?;
        let (input,_) = rec_separator(input)?;
        let (input,nmi_data_details) = separated_list(rec_separator, parse_nmi_data_details)(input)?;
        let (input,_) = rec_separator(input)?;
        let (_input,_) = EndOfData::parse(input)?;

        Ok(NEM12 {
            header,
            nmi_data_details
        })
    }
}

struct Parser<'a> {
    src: Input<'a>,
    iter: std::iter::Peekable<std::iter::Enumerate<str::Lines<'a>>>,
    data: Option<NEM12<'a>>,
    finished: bool,
    ctx: Mutex<(usize,)>,
    // ctx: (usize,),
}

impl <'a>Parser<'a> {

    fn new(src: Input<'a>) -> Self {
        let parser = Parser {
            src,
            iter: src.lines().enumerate().peekable(),
            data: None,
            finished: false,
            ctx:Mutex::new((0,)),
            // ctx:(0,),
        };

        parser
    }

    fn parse_line(&mut self) -> Result<Option<record::Kind>,(&'static str,usize,Err<Error<Input<'a>>>)> {
        if let Some((line_no,line)) = self.iter.next() {
            // println!("PARSING {}: '{}'",line_no,line);
            let res = alt((
                map(record::Header::parse, |o| Some(record::Kind::Header(o))),
                map(record::NMIDataDetails::parse, |o| {
                    let mut mutex = self.ctx.lock().unwrap();
                    mutex.0 = 1440usize / o.interval_length;
                    Some(record::Kind::NMIDataDetails(o))
                }),
                map(|input|record::IntervalData::parse(self.ctx.lock().unwrap().0,input),
                    |o| { Some(record::Kind::IntervalData(o)) }),
                map(record::IntervalEvent::parse, |o| Some(record::Kind::IntervalEvent(o))),
                map(record::B2BDetails::parse, |o| Some(record::Kind::B2BDetails(o))),
                map(record::EndOfData::parse, |o| Some(record::Kind::EndOfData(o))),
            ))(line.into())
                .map_err(|err| {
                    match err {
                        Err::Error(sentinel) => ("Error parsing line: ",line_no+1,Err::Error(sentinel)),
                        Err::Failure(sentinel) => ("Failed to parse line: ",line_no+1,Err::Failure(sentinel)),
                        Err::Incomplete(needed) => match needed {
                            Needed::Size(sz) => ("Failed to parse line: ",sz.into(),err),
                            Needed::Unknown => ("Failed to parse line: ",line_no+1,err)
                        }
                    }
                });
            match res {
                Ok(r) => Ok(r.1),
                Err(e) => Err(e),
            }

            // return Ok(Some(res.1));
        } else {
            match !self.finished {
                true => {
                    self.finished = true;
                    Ok(None)
                }
                false => Err(("Error: parser consumed all input",0,nom::Err::Incomplete(Needed::Unknown)))
            }
        }
    }
}

pub mod file {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;
        use pretty_assertions::{assert_eq};

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

        const DATADETAILS_ROWS_STR: &'static str = "200,NCDE001111,E1B1Q1E2,1,E1,N1,METSER123,Wh,15,\n\
        300,20031204,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,A,,,20031206011132,20031207011022\n\
        300,20031205,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,A,,,20031206011132,20031207011022\n\
        ";

        const NEM12_WITH_QUALITY: &'static str = "100,NEM12,200404201300,MDA1,Ret1\n\
        200,CCCC123456,E1,001,E1,N1,METSER123,kWh,30,\n\
        300,20040417,18.023,19.150,17.592,24.155,18.568,22.304,19.222,19.032,19.090,22.237,24.350,22.274,20.193,16.615,19.575,20.391,16.459,20.527,21.438,19.327,21.424,16.656,17.616,18.416,16.666,19.961,18.120,18.023,18.588,21.759,17.841,19.548,18.486,21.391,15.656,16.634,16.377,14.246,17.451,15.742,18.038,18.470,14.936,17.987,15.751,19.750,16.202,14.733,V,,,20040418203500,20040419003500\n\
        400,1,20,F14,76,\n\
        400,21,24,A,,\n\
        400,25,48,S14,1,\n\
        900\n\
        ";

        #[test]
        fn get_nmi_data_details_parser() {
            let (input,nmi_data_details) = record::NMIDataDetails::parse(DATADETAILS_ROWS_STR.into()).unwrap();
            
            let capacity = 1440 / nmi_data_details.interval_length;
            let (input,_) = preceded(rec_separator,|i|{record::IntervalData::parse(capacity, i)})(input).unwrap();
            let (input,_) = preceded(rec_separator,|i|{record::IntervalData::parse(capacity, i)})(input).unwrap();
            assert_eq!(input.into_fragment(),"\n");

            let (input, nmi_data_details) = parse_nmi_data_details(DATADETAILS_ROWS_STR.into()).unwrap();
            // TODO: Write test to compare output
        }

        #[test]
        fn get_nmi_with_data_quality() {
            let nmi_data_details = record::NMIDataDetails {
                nmi: "CCCC123456".into(),
                nmi_configuration: "E1".into(),
                register_id: "001".into(),
                nmi_suffix: "E1".into(),
                mdm_data_stream_id: Some("N1".into()),
                meter_serial_number: "METSER123".into(),
                uom: "KWH".into(),
                interval_length: 30usize,
                next_scheduled_read_date: None,
                interval_data_vec: Some(vec![IntervalData {
                    interval_date: NaiveDate::from_ymd(2004,04,17),
                    interval_value: vec![18.023, 19.15, 17.592, 24.155, 18.568, 22.304, 19.222, 19.032, 19.09, 22.237, 24.35, 22.274, 20.193, 16.615, 19.575, 20.391, 16.459, 20.527, 21.438, 19.327, 21.424, 16.656, 17.616, 18.416, 16.666, 19.961, 18.12, 18.023, 18.588, 21.759, 17.841, 19.548, 18.486, 21.391, 15.656, 16.634, 16.377, 14.246, 17.451, 15.742, 18.038, 18.47, 14.936, 17.987, 15.751, 19.75, 16.202, 14.733],
                    quality_method: "V".into(),
                    reason_code: None,
                    reason_description: None,
                    update_datetime: NaiveDateTime::parse_from_str("2004-04-18T20:35:00","%Y-%m-%dT%H:%M:%S").unwrap(),
                    msats_load_datetime: Some(NaiveDateTime::parse_from_str("2004-04-19T00:35:00","%Y-%m-%dT%H:%M:%S").unwrap()),
                    interval_events: Some(vec![
                        IntervalEvent::parse("400,1,20,F14,76,\n".into()).map(|(_,o)|o).unwrap(),
                        IntervalEvent::parse("400,21,24,A,,\n".into()).map(|(_,o)|o).unwrap(),
                        IntervalEvent::parse("400,25,48,S14,1,\n".into()).map(|(_,o)|o).unwrap(),
                    ])
                }]),
                b2b_details: None,
            };
            let nem12_test = NEM12 {
                header: record::Header::new(
                    "NEM12".into(),
                    NaiveDate::from_ymd(2004,04,20).and_hms(13,0,0),
                    "MDA1".into(),
                    "Ret1".into()
                ),
                nmi_data_details: vec![nmi_data_details],
            };
            let nem12_obj = NEM12::from_str(NEM12_WITH_QUALITY.into());
            assert_eq!(Ok(nem12_test),nem12_obj)
        }

        #[test]
        fn multiple_meters_from_str() {
            let _nem12_obj = NEM12::from_str(MULTIPLE_METERS_STR.into()).unwrap();
        }

        #[test]
        fn multiple_meters() {

            let mut parser = Parser::new(MULTIPLE_METERS_STR.into());

            for _ in 1..20 {
                parser.parse_line().unwrap();
            }

            assert_eq!(parser.parse_line(),Ok(Some(record::Kind::EndOfData(record::EndOfData{}))));
            assert_eq!(parser.parse_line(),Ok(None));
            assert_eq!(parser.parse_line(),Err(("Error: parser consumed all input",0,nom::Err::Incomplete(Needed::Unknown))));
        }

        #[test]
        fn nem12_from_str() {
            let _nem12_obj = NEM12::from_str(MULTIPLE_METERS_STR.into());
            if let Err(e) = _nem12_obj {
                println!("{:?}",e);
            }
            // _nem12_obj.unwrap();
        }
    }
}

pub mod record {
    use nom::multi::many_m_n;

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
    #[derive(Clone,Debug)]
    pub struct Header<'a> {
        format: Input<'a>,
        created: NaiveDateTime,
        from_participant: Input<'a>,
        to_participant: Input<'a>,
    }

    impl <'a>PartialEq for Header<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.format.into_fragment() == other.format.into_fragment() &&
            self.created == other.created &&
            self.from_participant.into_fragment() == other.from_participant.into_fragment() &&
            self.to_participant.into_fragment() == other.to_participant.into_fragment()
        }
    }

    impl <'a>Eq for Header<'a> { }

    impl <'a>Header<'a> {
        pub fn new(format:Input<'a>, created: NaiveDateTime, from_participant: Input<'a>, to_participant: Input<'a>) -> Self {
            Header {
                format,
                created,
                from_participant,
                to_participant
            }
        }

        pub fn parse(input: Input) -> IResult<Input,Header> {
            let (input, _) = tag("100,")(input)?;
            let (input, format) = alt((tag("NEM12"),tag("NEM13")))(input)?;
            let (input, _) = tag(",")(input)?;
            let (input, created) = datetime_12(input)?;
            let (input, _) = tag(",")(input)?;
            let (input, from_participant) = section_of_max_length(alphanumeric1,10)(input)?;
            let (input, _) = tag(",")(input)?;
            let (input, to_participant) = section_of_max_length(alphanumeric1,10)(input)?;
    
            let header = Header::new(
                format,
                created,
                from_participant,
                to_participant
            );
    
            Ok((input,header))
        }
    }

    // NMI data details record (200)
    #[derive(Clone,Debug)]
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
        pub interval_data_vec: Option<Vec<IntervalData<'a>>>,
        pub b2b_details: Option<Vec<B2BDetails<'a>>>,
    }

    impl <'a>PartialEq for NMIDataDetails<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.nmi.into_fragment() == other.nmi.into_fragment() &&
            self.nmi_configuration.into_fragment() == other.nmi_configuration.into_fragment() &&
            self.register_id.into_fragment() == other.register_id.into_fragment() &&
            self.nmi_suffix.into_fragment() == other.nmi_suffix.into_fragment() &&
            self.mdm_data_stream_id.map(|o| o.into_fragment()) == other.mdm_data_stream_id.map(|o| o.into_fragment()) &&
            self.meter_serial_number.into_fragment() == other.meter_serial_number.into_fragment() &&
            self.uom.into_fragment().to_owned().to_lowercase() == other.uom.into_fragment().to_owned().to_lowercase() &&
            self.interval_length == other.interval_length &&
            self.next_scheduled_read_date == other.next_scheduled_read_date &&
            self.interval_data_vec == other.interval_data_vec &&
            self.b2b_details == other.b2b_details
        }
    }

    impl <'a>Eq for NMIDataDetails<'a> { }

    impl <'a>NMIDataDetails<'a> {
        pub fn parse(input: Input) -> IResult<Input,NMIDataDetails> {
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
            let (input, interval_length) = section_of_exact_length(digit1, 2)(input).map(|(input,val)| (input,val.parse::<usize>().unwrap()))?;
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
    
            // let interval_data_length = 1440usize / interval_length;
            // let (input, interval_data_vec) = separated_list0(
            //     rec_separator, 
            //     |input| record::IntervalData::parse(interval_data_length, input)
            // )(input)?;
    
            let nmi_data_details  = NMIDataDetails {
                nmi,
                nmi_configuration,
                register_id,
                nmi_suffix,
                mdm_data_stream_id,
                meter_serial_number,
                uom,
                interval_length,
                next_scheduled_read_date,
                interval_data_vec: None,
                b2b_details: None,
            };
    
            Ok((input,nmi_data_details))
            }
    }

    // Interval data record (300)
    #[derive(Clone,Debug)]
    pub struct IntervalData<'a> {
        pub interval_date: NaiveDate,
        pub interval_value: Vec<f64>,
        pub quality_method: Input<'a>,
        pub reason_code: Option<Input<'a>>,
        pub reason_description: Option<Input<'a>>,
        pub update_datetime: NaiveDateTime,
        pub msats_load_datetime: Option<NaiveDateTime>,
        pub interval_events: Option<Vec<IntervalEvent<'a>>>
    }

    impl <'a>PartialEq for IntervalData<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.interval_date == other.interval_date &&
            self.interval_value == other.interval_value &&
            self.quality_method.into_fragment() == other.quality_method.into_fragment() &&
            self.reason_code.map(|o| o.into_fragment()) == other.reason_code.map(|o| o.into_fragment()) &&
            self.reason_description.map(|o| o.into_fragment()) == other.reason_description.map(|o| o.into_fragment()) &&
            self.update_datetime == other.update_datetime &&
            self.msats_load_datetime == other.msats_load_datetime &&
            self.interval_events == other.interval_events
        }
    }

    impl <'a>Eq for IntervalData<'a> { }

    impl <'a>IntervalData<'a> {
        pub fn parse(capacity: usize, input: Input<'a>) -> IResult<Input<'a>,IntervalData<'a>> {
            interval_data(capacity, input)
        }
    }

    fn interval_data<'a>(capacity: usize, input: Input<'a>) -> IResult<Input<'a>,IntervalData<'a>> {
        let (input, _) = tag("300,")(input)?;
        let (input, interval_date) = date_8(input)?;
        let (input, interval_value) = many_m_n(capacity,capacity,preceded(tag(","),double))(input)?;

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
        let (input, msats_load_datetime) = opt(datetime_14)(input)?;

        // // Get Event Codes (400 recs)
        // let (input,_) = rec_separator(input)?;
        // let (input,interval_events): (Input,Vec<record::IntervalEvent>) = separated_list0(rec_separator,record::IntervalEvent::parse)(input)?;

        let interval_data = IntervalData {
            interval_date,
            interval_value,
            quality_method,
            reason_code,
            reason_description,
            update_datetime,
            msats_load_datetime,
            interval_events: None
        };

        Ok((input,interval_data))
    }

    // Interval event record (400)
    #[derive(Clone,Debug)]
    pub struct IntervalEvent<'a> {
        pub start_interval: Input<'a>,
        pub end_interval: Input<'a>,
        pub quality_method: Input<'a>,
        pub reason_code: Option<Input<'a>>,
        pub reason_description: Option<Input<'a>>,
    }

    impl <'a>PartialEq for IntervalEvent<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.start_interval.into_fragment() == other.start_interval.into_fragment() &&
            self.end_interval.into_fragment() == other.end_interval.into_fragment() &&
            self.quality_method.into_fragment() == other.quality_method.into_fragment() &&
            self.reason_code.map(|o| o.into_fragment()) == other.reason_code.map(|o| o.into_fragment()) &&
            self.reason_description.map(|o| o.into_fragment()) == other.reason_description.map(|o| o.into_fragment())
        }
    }

    impl <'a>Eq for IntervalEvent<'a> { }

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
        let (input, reason_code) = optional_field(section_of_max_length(digit1,3),",")(input)?;
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
    #[derive(Clone,Debug)]
    pub struct B2BDetails<'a> {
        pub trans_code: Input<'a>,
        pub ret_service_order: Input<'a>,
        pub read_datetime: NaiveDateTime,
        pub index_read: Input<'a>,
    }

    impl <'a>PartialEq for B2BDetails<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.trans_code.into_fragment() == other.trans_code.into_fragment() &&
            self.ret_service_order.into_fragment() == other.ret_service_order.into_fragment() &&
            self.read_datetime == other.read_datetime &&
            self.index_read.into_fragment() == other.index_read.into_fragment()
        }
    }

    impl <'a>Eq for B2BDetails<'a> { }

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
        let (input, _) = multispace0(input)?; // TODO: Should this be removed?
        Ok((input,EndOfData {}))
    }

    #[cfg(test)]
    mod tests {
        use std::borrow::Borrow;

        use super::{record,Input};
        use nom::error;
        use chrono::{NaiveDate};
    
        #[test]
        fn header_100() {
            let date = NaiveDate::from_ymd(2004,5,1).and_hms(11, 35, 0);
            let header = record::Header::new (
                "NEM12".into(),
                date.clone(),
                "MDA1".into(),
                "Ret1".into()
            );
    
            let raw = "100,NEM12,200405011135,MDA1,Ret1\n";
    
            let res = record::Header::parse(raw.into());
    
            assert_eq!(res.map(|(r,v)| (r.into_fragment(),v)),
                Ok(("\n",header))
            );
    
            let header = record::Header::new (
                "NEM12".into(),
                date.clone().into(),
                "0123456789".into(),
                "Ret1".into()
            );
    
            let raw = "100,NEM12,200405011135,0123456789,Ret1\n";
    
            let res = record::Header::parse(raw.into()); /* {
                Ok(o) => o,
                Err(e) => { println!("{:?}",e); panic!("Failed") }
            }; */
    
            assert_eq!(res.map(|(r,v)| (r.into_fragment(),v)),
                Ok(("\n",header))
            );
    
            let raw = "100,NEM12,200405011135,12345678910,Ret1\n";
    
            let res = match record::Header::parse(raw.into())
            .map(|(r,v)| (r.into_fragment(),v)) {
                Ok(o) => { println!("{:?}",o); panic!("Failed") },
                Err(nom::Err::Error(e)) => { (e.input.into_fragment(),error::ErrorKind::Verify) }, //NOTE: must accomodate custom errors somehow
                Err(nom::Err::Incomplete(_)) |
                Err(nom::Err::Failure(_)) => panic!("This should never happen")
            };
    
            assert_eq!(res, ("12345678910,Ret1\n",error::ErrorKind::Verify));
        }
    
        #[test]
        fn nmi_data_details_200() {
            let nmi_data_details = record::NMIDataDetails {
                nmi: "VABD000163".into(),
                nmi_configuration: "E1Q1".into(),
                register_id: "1".into(),
                nmi_suffix: "E1".into(),
                mdm_data_stream_id: Some("N1".into()),
                meter_serial_number: "METSER123".into(),
                uom: "KWH".into(),
                interval_length: 30usize,
                next_scheduled_read_date: None,
                interval_data_vec: None,
                b2b_details: None,
            };
    
            let raw = "200,VABD000163,E1Q1,1,E1,N1,METSER123,KWH,30,\n";
    
            let res = record::NMIDataDetails::parse(raw.into());
    
            assert_eq!(res.map(|(i,v)|(i.into_fragment(),v)),Ok(("\n",nmi_data_details)));
    
            let raw = "200,VABD000163,E1Q1,1,E1,N1,METSER123,kWh,30,1234\n";
    
            let res = record::NMIDataDetails::parse(raw.into());
    
            assert_eq!(res.map(|(i,v)|(i.into_fragment(),v)).map_err(|e| {
                match e {
                    nom::Err::Incomplete(e)=> nom::Err::Incomplete(e),
                    nom::Err::Error(e) => nom::Err::Error(e.input.into_fragment()),
                    nom::Err::Failure(e) => nom::Err::Failure(e.input.into_fragment()),
                }
            }),Err(nom::Err::Error("1234\n")));
        }
    
        #[test]
        fn interval_data_300() {
            let interval_data = record::IntervalData {
                interval_date: NaiveDate::from_ymd(2004, 2, 1),
                interval_value: vec![1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111],
                quality_method: "A".into(),
                reason_code: None,
                reason_description: None,
                update_datetime: NaiveDate::from_ymd(2004, 2, 2).and_hms(12, 0, 25),
                msats_load_datetime: Some(NaiveDate::from_ymd(2004, 2, 2).and_hms(14, 25, 16)),
                interval_events: None,
            };
    
            let raw = "300,20040201,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,1.111,A,,,20040202120025,20040202142516";
    
            let res = record::IntervalData::parse(48, raw.into());
    
            assert_eq!(res.map(|(i,v)|(i.into_fragment(),v)),Ok(("",interval_data)));
        }
    
        #[test]
        fn interval_event_400() {
            let interval_event = record::IntervalEvent {
                start_interval: "1".into(),
                end_interval: "20".into(),
                quality_method: "F14".into(),
                reason_code: Some("76".into()),
                reason_description: None,
            };
    
            let raw = "400,1,20,F14,76,\n";
            let res = record::IntervalEvent::parse(raw.into());
            assert_eq!(res.map(|(i,v)|(i.into_fragment(),v)),Ok(("\n",interval_event)));

            let interval_event = record::IntervalEvent {
                start_interval: "25".into(),
                end_interval: "48".into(),
                quality_method: "S14".into(),
                reason_code: Some("1".into()),
                reason_description: None,
            };
    
            let raw = "400,25,48,S14,1,\n";
            let res = record::IntervalEvent::parse(raw.into());
            assert_eq!(res.map(|(i,v)|(i.into_fragment(),v)),Ok(("\n",interval_event)));

            let interval_event = record::IntervalEvent {
                start_interval: "21".into(),
                end_interval: "24".into(),
                quality_method: "A".into(),
                reason_code: None,
                reason_description: None,
            };
    
            let raw = "400,21,24,A,,\n";
            let res = record::IntervalEvent::parse(raw.into());
            assert_eq!(res.map(|(r,v)| (r.into_fragment(),v)),Ok(("\n",interval_event)));
        }
    
        #[test]
        fn b2b_details_500() {
            let interval_event = record::B2BDetails {
                trans_code: "S".into(),
                ret_service_order: "RETNSRVCEORD1".into(),
                read_datetime: NaiveDate::from_ymd(2003,12,20).and_hms(15,45,0),
                index_read: "001123.5".into(),
            };
    
            let raw = "500,S,RETNSRVCEORD1,20031220154500,001123.5\n";
            let res = record::B2BDetails::parse(raw.into());
            assert_eq!(res.map(|(r,v)| (r.into_fragment(),v)),Ok(("\n",interval_event)));
        }
    
        #[test]
        fn end_of_data_900() {
            let end_of_data = record::EndOfData {};
    
            let raw = "900\n";
            let res = record::EndOfData::parse(raw.into());
            assert_eq!(res.map(|(r,v)| (r.into_fragment(),v)),Ok(("",end_of_data)));
        }
    }
}