//This Represents the type of piece
use std::io::{self, Write};
use std::fmt;

const MIN_DIM: usize = 6;
const MAX_DIM: usize = 12;

// Represents the type of piece
#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceType {
    Developer,    // Jumps, captures by jumping over to an empty square
    Designer,     // L-shape, captures by landing on them
    ProductOwner, // 1 square, captures by landing on them
}

// Represents the player's color
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerColor {
    White,
    Black,
}

impl PlayerColor {
    fn opponent(&self) -> PlayerColor {
        match self {
            PlayerColor::White => PlayerColor::Black,
            PlayerColor::Black => PlayerColor::White,
        }
    }
}

// Represents a single chess piece
#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    piece_type: PieceType,
    color: PlayerColor,
}

impl Piece {
    fn new(piece_type: PieceType, color: PlayerColor) -> Self {
        Piece { piece_type, color }
    }
}

// Display trait for Piece to show Unicode characters
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match (&self.piece_type, &self.color) {
            (PieceType::ProductOwner, PlayerColor::White) => '♔', // White PO
            (PieceType::Developer, PlayerColor::White)    => '♖', // White Dev
            (PieceType::Designer, PlayerColor::White)     => '♘', // White Des
            (PieceType::ProductOwner, PlayerColor::Black) => '♚', // Black PO
            (PieceType::Developer, PlayerColor::Black)    => '♜', // Black Dev
            (PieceType::Designer, PlayerColor::Black)     => '♞', // Black Des
        };
        write!(f, "{}", symbol)
    }
}

// Represents a square on the board
type Square = Option<Piece>;

// Stores details about a potential move
// (target_row, target_col, is_capture, Option<(jumped_piece_row, jumped_piece_col)> for Developer)
#[derive(Debug, Clone, Copy)]
struct MoveDetail {
    to_r: usize,
    to_c: usize,
    is_capture: bool,
    jumped_piece_coord: Option<(usize, usize)>, // Only for Developer captures
}


// Represents the game board
struct Board {
    grid: Vec<Vec<Square>>,
    width: usize,
    height: usize,
}

impl Board {
    fn new(width: usize, height: usize) -> Self {
        let mut board = Board {
            grid: vec![vec![None; width]; height],
            width,
            height,
        };
        board.setup_pieces();
        board
    }

    fn setup_pieces(&mut self) {
        for r in 0..self.height {
            for c in 0..self.width {
                self.grid[r][c] = None;
            }
        }
        if self.width >= 1 { self.grid[0][0] = Some(Piece::new(PieceType::ProductOwner, PlayerColor::White)); }
        if self.width >= 2 { self.grid[0][1] = Some(Piece::new(PieceType::Developer, PlayerColor::White)); }
        if self.width >= 3 { self.grid[0][2] = Some(Piece::new(PieceType::Designer, PlayerColor::White)); }

        let top_row = self.height - 1;
        if self.width >= 1 { self.grid[top_row][self.width - 1] = Some(Piece::new(PieceType::ProductOwner, PlayerColor::Black));}
        if self.width >= 2 { self.grid[top_row][self.width - 2] = Some(Piece::new(PieceType::Developer, PlayerColor::Black));}
        if self.width >= 3 { self.grid[top_row][self.width - 3] = Some(Piece::new(PieceType::Designer, PlayerColor::Black));}
    }

    fn display(&self, selected_square: Option<(usize, usize)>, available_moves: &Option<Vec<MoveDetail>>) {
        println!();
        print!("   ");
        for c in 0..self.width { print!(" {} ", (b'A' + c as u8) as char); }
        println!();
        print!("  +-"); for _ in 0..self.width { print!("--"); } println!("+");

        for r_rev in 0..self.height {
            let r = self.height - 1 - r_rev;
            print!("{:2}|", r + 1);
            for c in 0..self.width {
                let is_selected = selected_square.map_or(false, |(sel_r, sel_c)| sel_r == r && sel_c == c);
                let mut move_char = ' ';

                if let Some(moves) = available_moves {
                    for move_detail in moves {
                        if move_detail.to_r == r && move_detail.to_c == c {
                            move_char = if move_detail.is_capture { '•' } else { '.' };
                            break;
                        }
                    }
                }
                
                let square_content = match self.grid[r][c] {
                    Some(piece) => format!("{}", piece),
                    None => format!("{}", move_char),
                };

                if is_selected { print!("[{}]", square_content); } 
                else { print!(" {} ", square_content); }
            }
            println!("|");
        }
        print!("  +-"); for _ in 0..self.width { print!("--"); } println!("+");
        println!();
    }

    fn get_piece(&self, r: usize, c: usize) -> Option<Piece> {
        if r < self.height && c < self.width {
            self.grid[r][c]
        } else {
            None
        }
    }

    // Calculate valid moves for a piece at (start_r, start_c)
    fn calculate_valid_moves(&self, start_r: usize, start_c: usize, piece: Piece) -> Vec<MoveDetail> {
        let mut moves = Vec::new();
        match piece.piece_type {
            PieceType::ProductOwner => {
                // Moves one square in any direction (8 directions)
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 { continue; } // Skip self

                        let to_r_signed = start_r as isize + dr;
                        let to_c_signed = start_c as isize + dc;

                        if to_r_signed >= 0 && to_r_signed < self.height as isize &&
                           to_c_signed >= 0 && to_c_signed < self.width as isize {
                            let to_r = to_r_signed as usize;
                            let to_c = to_c_signed as usize;

                            match self.grid[to_r][to_c] {
                                Some(target_piece) => {
                                    if target_piece.color != piece.color { // Opponent piece
                                        moves.push(MoveDetail { to_r, to_c, is_capture: true, jumped_piece_coord: None });
                                    }
                                    // else: friendly piece, cannot move
                                }
                                None => { // Empty square
                                    moves.push(MoveDetail { to_r, to_c, is_capture: false, jumped_piece_coord: None });
                                }
                            }
                        }
                    }
                }
            }
            PieceType::Designer => {
                // L-shape moves (2 in one dir, 1 perpendicular)
                let l_moves: [(isize, isize); 8] = [
                    (1, 2), (1, -2), (-1, 2), (-1, -2),
                    (2, 1), (2, -1), (-2, 1), (-2, -1),
                ];
                for (dr, dc) in l_moves.iter() {
                    let to_r_signed = start_r as isize + dr;
                    let to_c_signed = start_c as isize + dc;

                    if to_r_signed >= 0 && to_r_signed < self.height as isize &&
                       to_c_signed >= 0 && to_c_signed < self.width as isize {
                        let to_r = to_r_signed as usize;
                        let to_c = to_c_signed as usize;
                        match self.grid[to_r][to_c] {
                            Some(target_piece) => {
                                if target_piece.color != piece.color {
                                    moves.push(MoveDetail { to_r, to_c, is_capture: true, jumped_piece_coord: None });
                                }
                            }
                            None => {
                                moves.push(MoveDetail { to_r, to_c, is_capture: false, jumped_piece_coord: None });
                            }
                        }
                    }
                }
            }
            PieceType::Developer => {
                // Jumps up to 3 squares, any direction. Captures by jumping OVER to an EMPTY square.
                for dr_base in -1..=1 { // Direction vector row component
                    for dc_base in -1..=1 { // Direction vector col component
                        if dr_base == 0 && dc_base == 0 { continue; } // Skip no direction

                        for dist in 1..=3 { // Distance 1, 2, or 3
                            let to_r_signed = start_r as isize + dr_base * dist;
                            let to_c_signed = start_c as isize + dc_base * dist;

                            if to_r_signed < 0 || to_r_signed >= self.height as isize ||
                               to_c_signed < 0 || to_c_signed >= self.width as isize {
                                break; // Off board, stop this direction
                            }
                            let to_r = to_r_signed as usize;
                            let to_c = to_c_signed as usize;

                            // Target square must be empty for Developer
                            if self.grid[to_r][to_c].is_some() {
                                break; // Blocked by piece on target, stop this direction
                            }

                            // Check path for jumped piece
                            let mut jumped_piece_on_path: Option<(usize, usize)> = None;
                            let mut path_blocked_by_friendly = false;
                            let mut multiple_opponents_on_path = false;

                            if dist > 1 { // Only need to check path if jumping (dist 2 or 3)
                                for step in 1..dist { // Iterate over squares between start and target
                                    let path_r = (start_r as isize + dr_base * step) as usize;
                                    let path_c = (start_c as isize + dc_base * step) as usize;
                                    if let Some(path_piece) = self.grid[path_r][path_c] {
                                        if path_piece.color == piece.color {
                                            path_blocked_by_friendly = true;
                                            break;
                                        } else { // Opponent piece on path
                                            if jumped_piece_on_path.is_some() {
                                                multiple_opponents_on_path = true; // Second opponent on path
                                                break;
                                            }
                                            jumped_piece_on_path = Some((path_r, path_c));
                                        }
                                    }
                                }
                            }

                            if path_blocked_by_friendly || multiple_opponents_on_path {
                                continue; // Path is blocked, try next distance or direction
                            }
                            
                            // If target is empty and path is valid:
                            let is_capture = jumped_piece_on_path.is_some();
                            moves.push(MoveDetail { to_r, to_c, is_capture, jumped_piece_coord: jumped_piece_on_path });
                        }
                    }
                }
            }
        }
        moves
    }

    // Attempts to move a piece. Returns Ok(Option<Piece>) with captured piece if successful, Err(String) otherwise.
    fn move_piece(&mut self, from_r: usize, from_c: usize, to_r: usize, to_c: usize, current_player: PlayerColor, valid_moves: &[MoveDetail]) -> Result<Option<Piece>, String> {
        let moving_piece_option = self.get_piece(from_r, from_c);

        // Validation 1: Is there a piece at 'from'?
        let moving_piece = match moving_piece_option {
            Some(p) => p,
            None => return Err(format!("Invalid move: There is no piece at {}.", coords_to_algebraic(from_r, from_c, self.height))),
        };

        // Validation 2: Is it the current player's piece?
        if moving_piece.color != current_player {
            return Err("Invalid move: You can't move your opponent's piece.".to_string());
        }

        // Validation 3: Is 'to' different from 'from'?
        if from_r == to_r && from_c == to_c {
            return Err("Invalid move: Destination must be different from origin.".to_string());
        }

        // Validation 4: Is the move in the list of valid moves for the selected piece?
        let move_detail = valid_moves.iter().find(|m| m.to_r == to_r && m.to_c == to_c);
        
        let valid_move_info = match move_detail {
            Some(m_info) => m_info,
            None => return Err(format!("Invalid move: {} can't move to {}.", moving_piece, coords_to_algebraic(to_r, to_c, self.height))),
        };

        // Perform the move
        self.grid[from_r][from_c] = None; // Remove piece from original square
        let mut captured_piece_details: Option<Piece> = None;

        if valid_move_info.is_capture {
            match moving_piece.piece_type {
                PieceType::Developer => {
                    // Developer captures by jumping over, target square is empty.
                    // The piece to remove is at valid_move_info.jumped_piece_coord.
                    if let Some((jumped_r, jumped_c)) = valid_move_info.jumped_piece_coord {
                        captured_piece_details = self.grid[jumped_r][jumped_c].take(); // Take the jumped piece
                    } else {
                        // This should not happen if is_capture is true for Developer based on calculate_valid_moves
                        return Err("Internal error: Developer capture indicated but no jumped piece coordinate.".to_string());
                    }
                }
                PieceType::Designer | PieceType::ProductOwner => {
                    // These pieces capture by landing on the opponent's piece.
                    captured_piece_details = self.grid[to_r][to_c].take(); // Take the piece at the destination
                }
            }
        }
        
        self.grid[to_r][to_c] = Some(moving_piece); // Place moving piece at destination
        Ok(captured_piece_details)
    }
}

struct GameState {
    board: Board,
    current_player: PlayerColor,
    selected_square_coords: Option<(usize, usize)>,
    available_moves_for_selected: Option<Vec<MoveDetail>>,
    game_over: bool,
    winner: Option<PlayerColor>,
}

impl GameState {
    fn new(width: usize, height: usize) -> Self {
        GameState {
            board: Board::new(width, height),
            current_player: PlayerColor::White,
            selected_square_coords: None,
            available_moves_for_selected: None,
            game_over: false,
            winner: None,
        }
    }

    fn display_turn_info(&self) {
        if self.game_over {
            if let Some(winner) = self.winner {
                println!("{:?} wins! 🎉", winner);
                println!("Type \"restart\" to play again or \"exit\" to leave.");
            } else {
                println!("Game over! It's a draw (somehow?)."); // Should not happen with PO capture rule
            }
        } else {
            println!("Turn: {:?}", self.current_player);
        }
    }

    fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
        self.selected_square_coords = None;
        self.available_moves_for_selected = None;
    }

    fn select_piece(&mut self, r: usize, c: usize) -> Result<(), String> {
        if self.game_over { return Err("The game is over.".to_string()); }

        match self.board.get_piece(r, c) {
            Some(piece) => {
                if piece.color == self.current_player {
                    self.selected_square_coords = Some((r, c));
                    let moves = self.board.calculate_valid_moves(r, c, piece);
                    if !moves.is_empty() {
                        print!("Selected: {} at {}. Available moves: ", piece, coords_to_algebraic(r, c, self.board.height));
                        for (i, m) in moves.iter().enumerate() {
                            print!("{}", coords_to_algebraic(m.to_r, m.to_c, self.board.height));
                            if i < moves.len() - 1 { print!(", "); }
                        }
                        println!();
                    } else {
                        println!("Selected: {} at {}. No available moves.", piece, coords_to_algebraic(r, c, self.board.height));
                    }
                    self.available_moves_for_selected = Some(moves);
                    Ok(())
                } else {
                    Err(format!("Invalid input: You cannot select a {} piece on {:?}'s turn.",
                        format!("{:?}", piece.color).to_lowercase(), self.current_player))
                }
            }
            None => Err(format!("Invalid input: There is no piece at {}.", coords_to_algebraic(r,c,self.board.height).to_uppercase())),
        }
    }
    
    fn attempt_move(&mut self, from_r: usize, from_c: usize, to_r: usize, to_c: usize) -> Result<(), String> {
        if self.game_over { return Err("The game is over. Type 'restart' or 'exit'.".to_string()); }

        // Use available_moves_for_selected if a piece was selected, otherwise calculate them now (direct move command)
        let current_valid_moves = if self.selected_square_coords == Some((from_r, from_c)) {
            self.available_moves_for_selected.clone().unwrap_or_else(|| {
                // Should ideally not happen if select was called first, but as a fallback:
                if let Some(p) = self.board.get_piece(from_r, from_c) {
                    if p.color == self.current_player {
                        self.board.calculate_valid_moves(from_r, from_c, p)
                    } else { vec![] }
                } else { vec![] }
            })
        } else { // If 'move' is called without 'select', or for a different piece
             if let Some(p) = self.board.get_piece(from_r, from_c) {
                if p.color == self.current_player {
                     self.board.calculate_valid_moves(from_r, from_c, p)
                } else {
                    // This case is handled by board.move_piece, but good to be explicit
                    return Err("Invalid move: You can't move your opponent's piece.".to_string());
                }
            } else {
                return Err(format!("Invalid move: There is no piece at {}.", coords_to_algebraic(from_r, from_c, self.board.height)));
            }
        };


        match self.board.move_piece(from_r, from_c, to_r, to_c, self.current_player, &current_valid_moves) {
            Ok(captured_piece_option) => {
                let moved_piece_symbol = self.board.get_piece(to_r, to_c).map_or('?', |p| format!("{}",p).chars().next().unwrap());
                print!("Moved {} from {} to {}.", moved_piece_symbol, coords_to_algebraic(from_r, from_c, self.board.height), coords_to_algebraic(to_r, to_c, self.board.height));
                if let Some(captured) = captured_piece_option {
                    print!(" Captured {}.", captured);
                    if captured.piece_type == PieceType::ProductOwner {
                        self.game_over = true;
                        self.winner = Some(self.current_player);
                        // Game over message will be handled by display_turn_info
                    }
                }
                println!();
                
                if !self.game_over {
                    self.switch_player();
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

fn algebraic_to_coords(s: &str, board_height: usize, board_width: usize) -> Result<(usize, usize), String> {
    if s.len() < 2 { return Err(format!("Invalid coordinate format: {}", s)); }
    let mut chars = s.chars();
    let col_char = chars.next().unwrap().to_ascii_uppercase();
    let row_str: String = chars.collect();
    let col_idx = (col_char as u8).wrapping_sub(b'A') as usize;
    let row_num = match row_str.parse::<usize>() {
        Ok(n) => n,
        Err(_) => return Err(format!("Invalid row number in coordinate: {}", s)),
    };
    if row_num == 0 || row_num > board_height { return Err(format!("Row number {} out of bounds (1-{}).", row_num, board_height)); }
    let row_idx = row_num - 1;
    if col_idx >= board_width { return Err(format!("Column {} out of bounds (A-{}).", col_char, (b'A' + board_width as u8 - 1) as char)); }
    Ok((row_idx, col_idx))
}

fn coords_to_algebraic(r: usize, c: usize, _board_height: usize) -> String {
    format!("{}{}", (b'A' + c as u8) as char, r + 1)
}

fn get_board_dimension(prompt: &str) -> usize {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<usize>() {
            Ok(val) if (MIN_DIM..=MAX_DIM).contains(&val) => return val,
            _ => println!("Invalid input. Please enter a number between {} and {}.", MIN_DIM, MAX_DIM),
        }
    }
}

fn main() {
    println!("Welcome to Unvoid Chess!");
    let board_width = get_board_dimension("Enter board width (6-12): ");
    let board_height = get_board_dimension("Enter board height (6-12): ");
    println!("Starting match on the ({} x {}) board...", board_width, board_height);
    
    let mut game_state = GameState::new(board_width, board_height);

    loop {
        game_state.board.display(game_state.selected_square_coords, &game_state.available_moves_for_selected);
        game_state.display_turn_info();
        
        if game_state.game_over {
            // Only allow restart or exit if game is over
        } else {
             print!("Type a command (type \"help\" for options):\n> ");
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        if parts.is_empty() { continue; }
        let command = parts[0].to_lowercase();

        if game_state.game_over && !["restart", "exit"].contains(&command.as_str()) {
            println!("Game is over. Type \"restart\" to play again or \"exit\" to leave.");
            continue;
        }

        match command.as_str() {
            "help" => { 
                println!("Available commands:");
                println!("  move <from> <to>    Move a piece (e.g. move B1 C3)");
                println!("  select <square>     Highlight piece (e.g. select B1)");
                println!("  restart             Restart the match");
                println!("  exit                Exit the game");
                println!("  help                Show this list");}
            "exit" => { println!("Exiting Unvoid Chess. Goodbye!"); break; }
            "restart" => {
                println!("Restarting match...");
                game_state = GameState::new(board_width, board_height);
            }
            "select" => {
                if parts.len() == 2 {
                    let sq_str = parts[1];
                    match algebraic_to_coords(sq_str, game_state.board.height, game_state.board.width) {
                        Ok((r, c)) => {
                            if let Err(e) = game_state.select_piece(r, c) { println!("{}", e); }
                        }
                        Err(_) => { // Use generic error from images for bad coord format
                             println!("Invalid input: {} is not a valid square on the board.", sq_str.to_uppercase());
                             println!("Please enter coordinates from A1 to {}{}.", 
                                (b'A' + game_state.board.width as u8 - 1) as char, game_state.board.height);
                        }
                    }
                } else {
                    println!("Invalid input: The 'select' command takes only one coordinate.");
                    println!("Usage: select <square>");
                    println!("Example: select C1");
                }
            }
            "move" => {
                if parts.len() == 3 {
                    let from_str = parts[1];
                    let to_str = parts[2];
                    match (algebraic_to_coords(from_str, game_state.board.height, game_state.board.width),
                           algebraic_to_coords(to_str, game_state.board.height, game_state.board.width)) {
                        (Ok((from_r, from_c)), Ok((to_r, to_c))) => {
                            if let Err(e) = game_state.attempt_move(from_r, from_c, to_r, to_c) {
                                println!("{}", e);
                            }
                        }
                        (Err(_), _) => println!("Invalid input: {} is not a valid 'from' square.", from_str.to_uppercase()),
                        (_, Err(_)) => println!("Invalid input: {} is not a valid 'to' square.", to_str.to_uppercase()),
                    }
                } else {
                    println!("Invalid input: The 'move' command requires <from> and <to> coordinates.");
                    println!("Usage: move <from_square> <to_square>");
                    println!("Example: move B1 C3");
                }
            }
            _ => {
                println!("Unknown command: {}", command);
                println!("Type \"help\" to see a list of valid commands.");
            }
        }
        println!();
    }
}

