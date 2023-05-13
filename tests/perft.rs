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
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 0), 1);
}

#[test]
fn kiwipete_1() {
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 1), 48);
}

#[test]
fn kiwipete_2() {
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 2), 2039);
}

#[test]
fn kiwipete_3() {
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 3), 97862);
}

#[test]
fn kiwipete_4() {
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 4), 4085603);
}

#[test]
fn kiwipete_5() {
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 5), 193690690);
}

#[test]
#[ignore]
fn kiwipete_6() {
    assert_eq!(perft(&Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap(), 6), 8031647685);
}

