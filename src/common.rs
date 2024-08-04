use nom::{
    bytes::complete::{take,tag},
    character::complete::newline,
    // end of streaming combinator imports
    multi::many1_count,
    IResult,
    combinator::{verify,peek,},
    error,
    InputTake, Compare, InputLength
};
use chrono::{NaiveDateTime,NaiveDate};

use std::fmt;
use std::str;
use nom_locate::LocatedSpan;

pub type Input<'a> = LocatedSpan<&'a str>;

pub fn rec_separator(input: Input) -> IResult<Input,usize> {
    many1_count(newline)(input)
}

pub fn section_of_max_length<'a, E: error::ParseError<Input<'a>>, F: Copy>(
    test: F,
    length: usize
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Input<'a>, E>
where
    F: Fn(Input<'a>) -> IResult<Input<'a>, Input<'a>, E>
{
    verify(test, move |s: &Input| (s.len() <= length) && (s.len() > 0))
}

pub fn section_of_exact_length<'a, E: error::ParseError<Input<'a>>, F: Copy>(
    test: F,
    length: usize
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Input<'a>, E>
where
    F: Fn(Input<'a>) -> IResult<Input<'a>, Input<'a>, E>
{
    verify(test, move |s: &Input| s.len() == length)
}

pub fn optional_field<I1, T, O, E1, F>(
    mut test: F,
    end_marker: T
) -> impl FnMut(I1) -> IResult<I1, Option<O>, E1>
where
    I1: fmt::Debug + Clone + InputTake + Compare<T>,
    T: InputLength + Clone + Copy,
    E1: fmt::Debug + error::ParseError<I1>,
    O: fmt::Debug,
    F: FnMut(I1) -> IResult<I1, O, E1>
{
    move |input: I1| {
        let i = input.clone();
        match test(input) {
            Ok(d) => Ok((d.0,Some(d.1))),
            Err(nom::Err::Error(_)) => {
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

pub fn datetime_14(input: Input) -> IResult<Input,NaiveDateTime> {
    let (input, date_time_str) = take(14usize)(input)?;

    let date_time: NaiveDateTime = match NaiveDateTime::parse_from_str(date_time_str.fragment(),"%Y%m%d%H%M%S") {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::Fail)))
    }?;

    Ok((input,date_time))
}

pub fn datetime_12(input: Input) -> IResult<Input,NaiveDateTime> {
    let (input, date_time_str) = take(12usize)(input)?;

    let date_time: NaiveDateTime = match NaiveDateTime::parse_from_str(date_time_str.fragment(),"%Y%m%d%H%M") {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::Fail)))
    }?;

    Ok((input,date_time))
}

pub fn date_8(input: Input) -> IResult<Input,NaiveDate> {
    let (input, date_time_str) = take(8usize)(input)?;

    let date_time: NaiveDate = match NaiveDate::parse_from_str(date_time_str.fragment(),"%Y%m%d") {
        Ok(r) => Ok(r),
        Err(_) => Err(nom::Err::Error(error::make_error(input,error::ErrorKind::Fail)))
    }?;

    Ok((input,date_time))
}

pub fn eof<I: InputLength + Copy, E: error::ParseError<I>>(input: I) -> IResult<I, I, E> {
    if input.input_len() == 0 {
      Ok((input, input))
    } else {
      Err(nom::Err::Error(E::from_error_kind(input, error::ErrorKind::Eof)))
    }
}