use bernt::{movegen::perft, position::Position};

#[test]
fn startpos_0() {
    assert_eq!(perft(&Position::startpos(), 0), 1);
}

#[test]
fn startpos_1() {
    assert_eq!(perft(&Position::startpos(), 1), 20);
}

#[test]
fn startpos_2() {
    assert_eq!(perft(&Position::startpos(), 2), 400);
}

#[test]
fn startpos_3() {
    assert_eq!(perft(&Position::startpos(), 3), 8902);
}

#[test]
fn startpos_4() {
    assert_eq!(perft(&Position::startpos(), 4), 197281);
}

#[test]
fn startpos_5() {
    assert_eq!(perft(&Position::startpos(), 5), 4865609);
}

#[test]
#[ignore]
fn startpos_6() {
    assert_eq!(perft(&Position::startpos(), 6), 119060324);
}

#[test]
#[ignore]
fn startpos_7() {
    assert_eq!(perft(&Position::startpos(), 7), 3195901860);
}

// See https://www.chessprogramming.org/Perft_Results#Position_2
#[test]
fn kiwipete_0() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            0
        ),
        1
    );
}

#[test]
fn kiwipete_1() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            1
        ),
        48
    );
}

#[test]
fn kiwipete_2() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            2
        ),
        2039
    );
}

#[test]
fn kiwipete_3() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            3
        ),
        97862
    );
}

#[test]
fn kiwipete_4() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            4
        ),
        4085603
    );
}

#[test]
#[ignore]
fn kiwipete_5() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            5
        ),
        193690690
    );
}

#[test]
#[ignore]
fn kiwipete_6() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            6
        ),
        8031647685
    );
}

// See https://www.chessprogramming.org/Perft_Results#Position_3
#[test]
fn pos3_0() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            0
        ),
        1
    );
}

#[test]
fn pos3_1() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            1
        ),
        14
    );
}

#[test]
fn pos3_2() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            2
        ),
        191
    );
}

#[test]
fn pos3_3() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            3
        ),
        2812
    );
}

#[test]
fn pos3_4() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            4
        ),
        43238
    );
}

#[test]
fn pos3_5() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            5
        ),
        674624
    );
}

#[test]
#[ignore]
fn pos3_6() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            6
        ),
        11030083
    );
}

#[test]
#[ignore]
fn pos3_7() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            7
        ),
        178633661
    );
}

#[test]
#[ignore]
fn pos3_8() {
    assert_eq!(
        perft(
            &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
            8
        ),
        3009794393
    );
}

// See https://www.chessprogramming.org/Perft_Results#Position_4
#[test]
fn pos4_0() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            0
        ),
        1
    );
}

#[test]
fn pos4_1() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            1
        ),
        6
    );
}

#[test]
fn pos4_2() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            2
        ),
        264
    );
}

#[test]
fn pos4_3() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            3
        ),
        9467
    );
}

#[test]
fn pos4_4() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            4
        ),
        422333
    );
}

#[test]
#[ignore]
fn pos4_5() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            5
        ),
        15833292
    );
}

#[test]
#[ignore]
fn pos4_6() {
    assert_eq!(
        perft(
            &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            6
        ),
        706045033
    );
}

// See https://www.chessprogramming.org/Perft_Results#Position_5
#[test]
fn pos5_0() {
    assert_eq!(
        perft(
            &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            0
        ),
        1
    );
}

#[test]
fn pos5_1() {
    assert_eq!(
        perft(
            &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            1
        ),
        44
    );
}

#[test]
fn pos5_2() {
    assert_eq!(
        perft(
            &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            2
        ),
        1486
    );
}

#[test]
fn pos5_3() {
    assert_eq!(
        perft(
            &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            3
        ),
        62379
    );
}

#[test]
fn pos5_4() {
    assert_eq!(
        perft(
            &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            4
        ),
        2103487
    );
}

#[test]
#[ignore]
fn pos5_5() {
    assert_eq!(
        perft(
            &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            5
        ),
        89941194
    );
}

// See https://www.chessprogramming.org/Perft_Results#Position_5
#[test]
fn pos6_0() {
    assert_eq!(
        perft(
            &Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
            .unwrap(),
            0
        ),
        1
    );
}

#[test]
fn pos6_1() {
    assert_eq!(
        perft(
            &Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
            .unwrap(),
            1
        ),
        46
    );
}

#[test]
fn pos6_2() {
    assert_eq!(
        perft(
            &Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
            .unwrap(),
            2
        ),
        2079
    );
}

#[test]
fn pos6_3() {
    assert_eq!(
        perft(
            &Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
            .unwrap(),
            3
        ),
        89890
    );
}

#[test]
fn pos6_4() {
    assert_eq!(
        perft(
            &Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
            .unwrap(),
            4
        ),
        3894594
    );
}

#[test]
#[ignore]
fn pos6_5() {
    assert_eq!(
        perft(
            &Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            )
            .unwrap(),
            5
        ),
        164075551
    );
}
