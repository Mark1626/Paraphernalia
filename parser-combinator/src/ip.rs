use std::str::FromStr;

use crate::parser::{
    ParseResult, Parser, any_char, left, match_literal, one_or_more, pair, pred, right,
    whitespace_wrap, zero_or_more,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PingResult {
    header: PingHeader,
    packets: Vec<IcmpPacket>,
    statistics: PingStatistics,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IcmpPacket {
    byte_count: i32,
    from: String,
    seq: i32,
    ttl: i32,
    time: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PingHeader {
    target_ip: String,      // 1.1.1.1
    resolved_ip: String,    // 1.1.1.1 but in parentheses
    icmp_payload_size: i32, // 56
    total_packet_size: i32, // 84
}

#[derive(Clone, Debug, PartialEq)]
pub struct PingStatistics {
    packets_transmitted: i32,
    packets_received: i32,
    packet_loss: f64,
    time: i32,
    rtt: Rtt,
}

#[derive(Clone, Debug, PartialEq)]
struct Rtt {
    min: f64,
    avg: f64,
    max: f64,
    mdev: f64,
}

// Collects a run of `allowed` chars and converts it with `FromStr`.
fn parse_run<'a, T, F>(allowed: F) -> impl Parser<'a, T>
where
    T: FromStr,
    F: Fn(&char) -> bool,
{
    move |input: &'a str| {
        let (rest, chars) = one_or_more(pred(any_char, &allowed)).parse(input)?;
        match chars.into_iter().collect::<String>().parse() {
            Ok(value) => Ok((rest, value)),
            Err(_) => Err(input),
        }
    }
}

fn integer<'a>(input: &'a str) -> ParseResult<'a, i32> {
    parse_run(|c: &char| c.is_ascii_digit()).parse(input)
}

fn decimal<'a>(input: &'a str) -> ParseResult<'a, f64> {
    parse_run(|c: &char| c.is_ascii_digit() || *c == '.').parse(input)
}

// A host or IP token: stops at whitespace, ':', '(' and ')'.
fn string<'a>(input: &'a str) -> ParseResult<'a, String> {
    parse_run(|c: &char| c.is_alphanumeric() || *c == '.' || *c == '-').parse(input)
}

// PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
fn ping_header<'a>() -> impl Parser<'a, PingHeader> {
    pair(
        right(match_literal("PING "), string),
        pair(
            right(match_literal(" ("), left(string, match_literal(") "))),
            pair(
                integer,
                right(
                    match_literal("("),
                    left(integer, match_literal(") bytes of data.")),
                ),
            ),
        ),
    )
    .map(
        |(target_ip, (resolved_ip, (icmp_payload_size, total_packet_size)))| PingHeader {
            target_ip,
            resolved_ip,
            icmp_payload_size,
            total_packet_size,
        },
    )
}

// 64 bytes from 1.1.1.1: icmp_seq=1 ttl=52 time=389 ms
fn icmp_packet<'a>() -> impl Parser<'a, IcmpPacket> {
    pair(
        integer,
        pair(
            right(match_literal(" bytes from "), string),
            pair(
                right(match_literal(": icmp_seq="), integer),
                pair(
                    right(match_literal(" ttl="), integer),
                    right(match_literal(" time="), left(decimal, match_literal(" ms"))),
                ),
            ),
        ),
    )
    .map(|(byte_count, (from, (seq, (ttl, time))))| IcmpPacket {
        byte_count,
        from,
        seq,
        ttl,
        time,
    })
}

// rtt min/avg/max/mdev = 47.129/115.880/388.750/112.515 ms
fn rtt<'a>() -> impl Parser<'a, Rtt> {
    pair(
        right(match_literal("rtt min/avg/max/mdev = "), decimal),
        pair(
            right(match_literal("/"), decimal),
            pair(
                right(match_literal("/"), decimal),
                right(match_literal("/"), left(decimal, match_literal(" ms"))),
            ),
        ),
    )
    .map(|(min, (avg, (max, mdev)))| Rtt {
        min,
        avg,
        max,
        mdev,
    })
}

// --- 1.1.1.1 ping statistics ---
// 8 packets transmitted, 7 received, 12.5% packet loss, time 7008ms
// rtt min/avg/max/mdev = 47.129/115.880/388.750/112.515 ms
fn ping_statistics<'a>() -> impl Parser<'a, PingStatistics> {
    let banner = right(
        match_literal("--- "),
        left(string, match_literal(" ping statistics ---")),
    );
    let counts = pair(
        left(integer, match_literal(" packets transmitted, ")),
        pair(
            left(integer, match_literal(" received, ")),
            pair(
                left(decimal, match_literal("% packet loss, time ")),
                left(integer, match_literal("ms")),
            ),
        ),
    );

    right(
        whitespace_wrap(banner),
        pair(whitespace_wrap(counts), whitespace_wrap(rtt())),
    )
    .map(
        |((packets_transmitted, (packets_received, (packet_loss, time))), rtt)| PingStatistics {
            packets_transmitted,
            packets_received,
            packet_loss,
            time,
            rtt,
        },
    )
}

pub fn ping_result<'a>() -> impl Parser<'a, PingResult> {
    pair(
        whitespace_wrap(ping_header()),
        pair(
            zero_or_more(whitespace_wrap(icmp_packet())),
            whitespace_wrap(ping_statistics()),
        ),
    )
    .map(|(header, (packets, statistics))| PingResult {
        header,
        packets,
        statistics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_parser() {
        assert_eq!(
            Ok((
                "",
                PingHeader {
                    target_ip: "1.1.1.1".to_string(),
                    resolved_ip: "1.1.1.1".to_string(),
                    icmp_payload_size: 56,
                    total_packet_size: 84,
                }
            )),
            ping_header().parse("PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.")
        );
    }

    #[test]
    fn packet_parser() {
        assert_eq!(
            Ok((
                "",
                IcmpPacket {
                    byte_count: 64,
                    from: "1.1.1.1".to_string(),
                    seq: 2,
                    ttl: 52,
                    time: 68.7,
                }
            )),
            icmp_packet().parse("64 bytes from 1.1.1.1: icmp_seq=2 ttl=52 time=68.7 ms")
        );
    }

    #[test]
    fn ping_result_parser() {
        let log = r#"
            PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
            64 bytes from 1.1.1.1: icmp_seq=1 ttl=52 time=389 ms
            64 bytes from 1.1.1.1: icmp_seq=2 ttl=52 time=68.7 ms
            --- 1.1.1.1 ping statistics ---
            3 packets transmitted, 2 received, 33.3% packet loss, time 2003ms
            rtt min/avg/max/mdev = 68.700/228.850/389.000/160.150 ms
            "#;
        let (rest, result) = ping_result().parse(log).unwrap();
        assert_eq!("", rest);
        assert_eq!(2, result.packets.len());
        assert_eq!(389.0, result.packets[0].time);
        assert_eq!(
            PingStatistics {
                packets_transmitted: 3,
                packets_received: 2,
                packet_loss: 33.3,
                time: 2003,
                rtt: Rtt {
                    min: 68.7,
                    avg: 228.85,
                    max: 389.0,
                    mdev: 160.15,
                },
            },
            result.statistics
        );
    }
}
