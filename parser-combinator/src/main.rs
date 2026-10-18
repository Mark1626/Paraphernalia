// Follow along of the parser combinator in this blog
// https://bodil.lol/parser-combinators/
// To convert the parser combinator in this blog into rust
// https://t3.at/blog/2026-10-17-parser-combinators

use parser_combinator::{ip, parser::Parser, xml};

fn main() {
    let doc = r#"
        <top label="Top">
            <semi-bottom label="Bottom"/>
            <middle>
                <bottom label="Another bottom"/>
            </middle>
        </top>"#;

    let d = xml::element().parse(doc);

    println!("Result {:?}", d);

    let ip_log = r#"
        PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
        64 bytes from 1.1.1.1: icmp_seq=1 ttl=52 time=389 ms
        64 bytes from 1.1.1.1: icmp_seq=2 ttl=52 time=68.7 ms
        64 bytes from 1.1.1.1: icmp_seq=3 ttl=52 time=47.1 ms
        64 bytes from 1.1.1.1: icmp_seq=4 ttl=52 time=48.6 ms
        64 bytes from 1.1.1.1: icmp_seq=5 ttl=52 time=85.8 ms
        64 bytes from 1.1.1.1: icmp_seq=6 ttl=52 time=84.8 ms
        64 bytes from 1.1.1.1: icmp_seq=7 ttl=52 time=87.4 ms
        --- 1.1.1.1 ping statistics ---
        8 packets transmitted, 7 received, 12.5% packet loss, time 7008ms
        rtt min/avg/max/mdev = 47.129/115.880/388.750/112.515 ms
        "#;

    let p = ip::ping_result().parse(ip_log);

    println!("Result {:?}", p);
}
