"""HoloCode v0.1: an executable experiment for holoscenic semantics."""

from .parser import parse
from .runtime import Runtime

__all__ = ["Runtime", "parse"]
__version__ = "0.1.0"

