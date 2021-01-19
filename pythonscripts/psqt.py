from chess_utils import *
import chess
import numpy as np
import torch
from torch import nn
import pytorch_lightning as pl
from utils import LambdaLayer

PIECE_FEATURE_MAP = {piece: index * 64 for (index, piece) in enumerate(iter(semantical_piece_order))}

PIECE_VALUES = {W_PAWN: 100 / 512, W_KNIGHT: 325 / 512, W_BISHOP: 350 / 512, W_ROOK: 550 / 512, W_QUEEN: 1000 / 512,
                W_KING: 0,
                B_PAWN: -100 / 512, B_KNIGHT: -325 / 512, B_BISHOP: -350 / 512, B_ROOK: -550 / 512,
                B_QUEEN: -1000 / 512, B_KING: 0}

FEATURES = 12 * 64 + 1
FEATURE_TEMPO = 12 * 64


def fill_features(matrix, at_index, fen):
    tokens = fen.split(" ")
    epd, tempo = tokens[0], int({"w": 1, "b": -1}[tokens[1]])
    matrix[at_index, FEATURE_TEMPO] = tempo
    board = chess.BaseBoard(board_fen=epd)
    for piece in semantical_piece_order:
        for sq in board.pieces(piece.piece_type, piece.color):
            matrix[at_index, PIECE_FEATURE_MAP[piece] + sq] = 1


def get_features(fen):
    res = np.zeros((1, FEATURES), dtype=np.int8)
    fill_features(res, 0, fen)
    return res


class PSQT(pl.LightningModule):
    def __init__(self, initialize_with_piece_values=True, loss=nn.MSELoss()):
        super().__init__()
        self.psqt = nn.Sequential(
            LambdaLayer(lambda x: x.type(torch.FloatTensor)),
            nn.Linear(FEATURES, 1),
            nn.Sigmoid()
        )
        if initialize_with_piece_values:
            self.initialize_piece_values()
        self.loss = loss

    def initialize_piece_values(self):
        weights = self.psqt.state_dict()['1.weight']
        for piece in semantical_piece_order:
            for sq in range(64):
                weights[0, PIECE_FEATURE_MAP[piece] + sq] += PIECE_VALUES[piece]

    def forward(self, x):
        return self.psqt(x)

    def predict_fen(self, fen):
        return self(torch.tensor(get_features(fen)))

    def training_step(self, batch, batch_idx):
        x, y = batch
        x_hat = self(x)
        loss = self.loss(x_hat, y)
        self.log('train_loss', loss, on_epoch=True, prog_bar=True)
        return loss

    def validation_step(self, batch, batch_idx):
        x, y = batch
        x_hat = self(x)
        loss = self.loss(x_hat, y)
        self.log('val_loss', loss, on_epoch=True, prog_bar=True)
        return loss

    def configure_optimizers(self):
        return torch.optim.Adam(self.parameters())

    def to_rust(self, sc=2 ** 17, shift=2 ** 9):
        params = list(self.parameters())
        weights = params[0].detach().numpy()[0]
        bias = params[1].detach().numpy()[0]

        def scale(w):
            return round(sc * w)

        def print_psqt():
            res = np.zeros((15, 64))
            for piece in semantical_piece_order:
                for sq in range(64):
                    res[PIECE_INDEX_MAP[piece], sq] = scale(weights[PIECE_FEATURE_MAP[piece] + sq])
            res_str = "["
            for i in range(len(res)):
                res_str += "["
                for j in range(len(res[i])):
                    res_str += str(int(res[i, j])) + ", "
                res_str += "], "
            res_str += "]"
            return res_str

        rust_psqt = "pub const PSQT: [[i32; 64]; {}] = {};".format(PIECE_MAX_INDEX, print_psqt())
        rust_tempo_bonus = "pub const TEMPO_BONUS: i32 = {};".format(scale(weights[-1]))
        rust_bias = "pub const BIAS: i32 = {};".format(scale(bias))
        rust_shift = "pub const DIV: i32 = {};".format(shift)
        print(rust_psqt)
        print(rust_tempo_bonus)
        print(rust_bias)
        print(rust_shift)
