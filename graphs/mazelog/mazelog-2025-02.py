# Author: Nimalan M (@mark1626)
# Solution to http://www.mazelog.com/show?1AK using a LogicT monad

from dataclasses import dataclass
from enum import Enum, auto
from typing import Optional, Tuple, TypeVar, Generic, Callable, List, Iterator, Set
from functools import lru_cache

T = TypeVar('T')
U = TypeVar('U')

class Piece(Enum):
    KING = auto()
    KNIGHT = auto()
    BISHOP = auto()
    ROOK = auto()
    WILDCARD = auto()

@dataclass(frozen=True)
class Board:
    size: int
    current_position: Tuple[int, int]
    current_piece: Piece
    prev_piece: Optional[Piece] = None

class LogicT(Generic[T]):
    def __init__(self, thunk: Callable[[], Iterator[T]]) -> None:
        self.thunk = thunk

    def __call__(self) -> Iterator[T]:
        return self.thunk()

    @staticmethod
    def unit(value: T) -> 'LogicT[T]':
        return LogicT(lambda: iter([value]))

    @staticmethod
    def mzero() -> 'LogicT[T]':
        return LogicT(lambda: iter([]))

    def bind(self, f: Callable[[T], 'LogicT[U]']) -> 'LogicT[U]':
        def thunk() -> Iterator[U]:
            for a in self():
                for b in f(a)():
                    yield b
        return LogicT(thunk)

    def plus(self, other: 'LogicT[T]') -> 'LogicT[T]':
        def thunk() -> Iterator[T]:
            for x in self():
                yield x
            for x in other():
                yield x
        return LogicT(thunk)

    def observe(self) -> Iterator[T]:
        return self()

    @staticmethod
    def guard(condition: bool) -> 'LogicT[None]':
        return LogicT.unit(None) if condition else LogicT.mzero()

@dataclass(frozen=True)
class MazeState:
    position: Tuple[int, int]
    piece: Piece
    prev_piece: Optional[Piece] = None
    last_non_wildcard: Optional[Piece] = None
    depth: int = 0

    def __hash__(self):
        return hash((self.position, self.piece, self.prev_piece, self.last_non_wildcard, self.depth))

# Cache for storing visited states to avoid cycles
VISITED_STATES: Set[Tuple[int, int, Piece, Optional[Piece], Optional[Piece]]] = set()

@lru_cache(maxsize=None)
def get_valid_moves(piece: Piece, x: int, y: int, prev_piece: Optional[Piece] = None, last_non_wildcard: Optional[Piece] = None) -> List[Tuple[int, int]]:
    if piece == Piece.WILDCARD:
        if prev_piece == Piece.WILDCARD:
            if last_non_wildcard is None:
                return get_valid_moves(Piece.KING, x, y)
            return get_valid_moves(last_non_wildcard, x, y)
        elif prev_piece is None:
            return get_valid_moves(Piece.KING, x, y)
        return get_valid_moves(prev_piece, x, y)
    
    moves = []
    
    if piece == Piece.KING:
        # King moves one step in any direction
        directions = [(dx, dy) for dx in [-1, 0, 1] for dy in [-1, 0, 1] if dx != 0 or dy != 0]
        moves.extend((x + dx, y + dy) for dx, dy in directions)
    
    elif piece == Piece.KNIGHT:
        # Knight can jump over pieces
        knight_moves = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1)
        ]
        moves.extend((x + dx, y + dy) for dx, dy in knight_moves)
    
    elif piece == Piece.BISHOP:
        # Bishop moves one step diagonally
        directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)]
        moves.extend((x + dx, y + dy) for dx, dy in directions)
    
    elif piece == Piece.ROOK:
        # Rook moves one step horizontally or vertically
        directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]
        moves.extend((x + dx, y + dy) for dx, dy in directions)
    
    return [(nx, ny) for nx, ny in moves if 0 <= nx < 6 and 0 <= ny < 6]

def make_move(state: MazeState, path: List[Tuple[int, int]]) -> LogicT[Tuple[MazeState, List[Tuple[int, int]]]]:
    def valid_moves() -> Iterator[Tuple[MazeState, List[Tuple[int, int]]]]:
        # Early termination conditions
        if state.depth > 30 or len(path) > 30:
            return
            
        x, y = state.position
        moves = get_valid_moves(state.piece, x, y, state.prev_piece, state.last_non_wildcard)
        
        # Get the previous position to prevent immediate backtracking
        prev_pos = path[-2] if len(path) > 1 else None
        
        for new_x, new_y in moves:
            # Skip if this move would immediately backtrack
            if prev_pos and (new_x, new_y) == prev_pos:
                continue
                
            new_piece = MAZE[new_x][new_y]
            new_path = path + [(new_x, new_y)]
            
            # Update last_non_wildcard
            new_last_non_wildcard = state.last_non_wildcard
            if state.piece != Piece.WILDCARD:
                new_last_non_wildcard = state.piece
            
            new_state = MazeState(
                position=(new_x, new_y),
                piece=new_piece,
                prev_piece=state.piece,
                last_non_wildcard=new_last_non_wildcard,
                depth=state.depth + 1
            )
            
            new_state_key = (new_state.position, new_state.piece, new_state.prev_piece, new_state.last_non_wildcard)
            if new_state_key not in VISITED_STATES:
                VISITED_STATES.add(new_state_key)
                yield new_state, new_path
    
    return LogicT(valid_moves)

def solve_maze() -> LogicT[List[Tuple[int, int]]]:
    def is_goal(state: MazeState) -> bool:
        return state.position == END_POSITION

    def solve_from(state: MazeState, path: List[Tuple[int, int]]) -> LogicT[List[Tuple[int, int]]]:
        if is_goal(state):
            return LogicT.unit(path)
        
        return make_move(state, path).bind(
            lambda state_and_path: solve_from(state_and_path[0], state_and_path[1])
        )

    # Clear the cache before starting a new solve
    VISITED_STATES.clear()
    
    initial_state = MazeState(
        position=START_POSITION,
        piece=MAZE[START_POSITION[0]][START_POSITION[1]],
        last_non_wildcard=None
    )
    
    return solve_from(initial_state, [START_POSITION])

def main():
    solutions = solve_maze().observe()
    found = False
    
    for path in solutions:
        found = True
        print("Path length:", len(path))
        
        # Print the path as sequence of indices
        indices = [pos[0] * 6 + pos[1] for pos in path]
        path_str = " -> ".join(str(idx+1) for idx in indices)
        print(path_str)
    
    if not found:
        print("No solution found!")

# 6x6 maze representation
MAZE = [
    [Piece.KING, Piece.BISHOP, Piece.ROOK, Piece.WILDCARD, Piece.BISHOP, Piece.BISHOP],
    [Piece.BISHOP, Piece.KNIGHT, Piece.BISHOP, Piece.WILDCARD, Piece.WILDCARD, Piece.WILDCARD],
    [Piece.BISHOP, Piece.WILDCARD, Piece.BISHOP, Piece.BISHOP, Piece.KNIGHT, Piece.WILDCARD],
    [Piece.BISHOP, Piece.KNIGHT, Piece.BISHOP, Piece.ROOK, Piece.BISHOP, Piece.KNIGHT],
    [Piece.KNIGHT, Piece.WILDCARD, Piece.BISHOP, Piece.BISHOP, Piece.ROOK, Piece.BISHOP],
    [Piece.BISHOP, Piece.ROOK, Piece.BISHOP, Piece.KNIGHT, Piece.WILDCARD, Piece.KING]
]

START_POSITION = (0, 0)
END_POSITION = (5, 5)

if __name__ == "__main__":
    main()
