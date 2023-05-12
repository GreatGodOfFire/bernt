use bernt::{movegen::perft, position::Position};

#[test]
fn startpos_0() {
    assert_eq!(perft(&Position::startpos(), 0).all, 1);
}

#[test]
fn startpos_1() {
    assert_eq!(perft(&Position::startpos(), 1).all, 20);
}

#[test]
fn startpos_2() {
    assert_eq!(perft(&Position::startpos(), 2).all, 400);
}

#[test]
fn startpos_3() {
    assert_eq!(perft(&Position::startpos(), 3).all, 8902);
}

#[test]
fn startpos_4() {
    assert_eq!(perft(&Position::startpos(), 4).all, 197281);
}

#[test]
fn startpos_5() {
    assert_eq!(perft(&Position::startpos(), 5).all, 4865609);
}

#[test]
fn startpos_6() {
    assert_eq!(perft(&Position::startpos(), 6).all, 119060324);
}
