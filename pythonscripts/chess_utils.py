from chess import PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK, Piece

W_PAWN, W_KNIGHT, W_BISHOP, W_ROOK, W_QUEEN, W_KING = Piece(PAWN, WHITE), Piece(KNIGHT, WHITE), Piece(BISHOP,
                                                                                                      WHITE), Piece(
    ROOK, WHITE), Piece(QUEEN, WHITE), Piece(KING, WHITE)
B_PAWN, B_KNIGHT, B_BISHOP, B_ROOK, B_QUEEN, B_KING = Piece(PAWN, BLACK), Piece(KNIGHT, BLACK), Piece(BISHOP,
                                                                                                      BLACK), Piece(
    ROOK, BLACK), Piece(QUEEN, BLACK), Piece(KING, BLACK)
semantical_piece_order = [W_PAWN, W_KNIGHT, W_BISHOP, W_ROOK, W_QUEEN, W_KING,
                          B_PAWN, B_KNIGHT, B_BISHOP, B_ROOK, B_QUEEN, B_KING]
# Piece Index map is how pieces are defined in Scam,
# see https://github.com/fabianvdW/Scam/blob/81f8f85bc4f52655b852f87be43546c7dfea6c8c/src/types.rs#L72-L84
PIECE_INDEX_MAP = {W_PAWN: 1, W_KNIGHT: 2, W_BISHOP: 3, W_ROOK: 4, W_QUEEN: 5, W_KING: 6,
                   B_PAWN: 9, B_KNIGHT: 10, B_BISHOP: 11, B_ROOK: 12, B_QUEEN: 13, B_KING: 14}
PIECE_MAX_INDEX = 15
