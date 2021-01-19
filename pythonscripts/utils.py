import torch
from pytorch_lightning import Callback


class LambdaLayer(torch.nn.Module):
    def __init__(self, lambd):
        super(LambdaLayer, self).__init__()
        self.lambd = lambd

    def forward(self, x):
        return self.lambd(x)


class MetricsCallback(Callback):
    def __init__(self):
        super().__init__()

    def on_epoch_end(self, trainer, pl_module):
        epoch = trainer.logged_metrics['epoch']
        val_loss = trainer.progress_bar_metrics['val_loss']
        train_loss = trainer.progress_bar_metrics['train_loss_epoch']
        print("Epoch {} - Loss {} - Val Loss: {}".format(epoch, train_loss, val_loss))
