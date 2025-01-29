(* Shape type definition *)
type shape = Diamond | Star | Triangle | Hexagon

(* Color type definition *)
type color = Green | Purple | Blue | Pink

(* Cell type definition *)
type cell = {
  shape: shape;
  color: color;
  circled: bool
}

(* Position type *)
type position = int * int

(* Maze type definition *)
type maze = {
  grid : cell option array array;
  start_point : position;
  end_point : position
}

type movement = Diagonal | Normal

type state = cell * movement * position array

(* Helper function to convert shape to string for display *)
let shape_to_string = function
  | Diamond -> "◆"
  | Star -> "★"
  | Triangle -> "▲"
  | Hexagon -> "⬡"

(* Get cell at specific coordinates *)
let get_cell maze cell_id =
  let (row, col) = cell_id in
  if row < 0 || row >= Array.length maze.grid ||
     col < 0 || col >= Array.length maze.grid.(0) then
    None
  else
    maze.grid.(row).(col)

(* Display cell at specific coordinates *)
let display_cell maze cell =
  let row, col = cell in
  match get_cell maze (row, col) with
  | None -> " "
  | Some cell -> 
      shape_to_string cell.shape

(* Display a path of moves *)
let display_path maze steps =
  String.concat " -> " (
    Array.to_list (Array.map (fun pos ->
      let (row, col) = pos in
      let index = row * 5 + col + 1 in
      Printf.sprintf "%d%s%s" 
        index
        (match get_cell maze pos with
         | Some cell -> display_cell maze pos
         | None -> "?")
        (match get_cell maze pos with
         | Some cell -> if cell.circled then "●" else ""
         | None -> "")
    ) steps)
  )

(* Display the maze *)
let display_maze maze =
  for i = 0 to Array.length maze.grid - 1 do
    for j = 0 to Array.length maze.grid.(0) - 1 do
      print_string (display_cell maze (i, j));
      print_string " "
    done;
    print_newline ()
  done

(* Position comparison *)
let pos_equal (p1: position) (p2: position) =
  fst p1 = fst p2 && snd p1 = snd p2

(* Check if a move is valid based on current position and movement type *)
let is_valid_move curr_pos next_pos movement =
  let (curr_row, curr_col) = curr_pos in
  let (next_row, next_col) = next_pos in
  let row_diff = abs (next_row - curr_row) in
  let col_diff = abs (next_col - curr_col) in
  match movement with
  | Diagonal -> row_diff = col_diff  (* Must move diagonally *)
  | Normal -> (row_diff = 0 && col_diff > 0) || (col_diff = 0 && row_diff > 0)  (* Must move horizontally or vertically *)

(* Get direction between two positions *)
let get_direction pos1 pos2 =
  let (row1, col1) = pos1 in
  let (row2, col2) = pos2 in
  (row2 - row1, col2 - col1)

(* Check if directions are opposite *)
let is_opposite_direction dir1 dir2 =
  let (dr1, dc1) = dir1 in
  let (dr2, dc2) = dir2 in
  let sign x = if x > 0 then 1 else if x < 0 then -1 else 0 in
  let sr1, sc1 = sign dr1, sign dc1 in
  let sr2, sc2 = sign dr2, sign dc2 in
  sr1 = -sr2 && sc1 = -sc2

(* Generate positions for multiple steps *)
let generate_positions row col movement max_steps prev_direction =
  let rec generate_direction dr dc steps acc =
    if steps > max_steps then acc
    else 
      let new_pos = (row + dr * steps, col + dc * steps) in
      generate_direction dr dc (steps + 1) (new_pos :: acc)
  in
  match movement with
  | Diagonal -> 
      (* Only diagonal moves *)
      List.concat [
        generate_direction (-1) (-1) 1 [];  (* up-left *)
        generate_direction (-1) 1 1 [];     (* up-right *)
        generate_direction 1 (-1) 1 [];     (* down-left *)
        generate_direction 1 1 1 []         (* down-right *)
      ]
  | Normal ->
      (* Only horizontal/vertical moves *)
      List.concat [
        generate_direction (-1) 0 1 [];     (* up *)
        generate_direction 1 0 1 [];        (* down *)
        generate_direction 0 (-1) 1 [];     (* left *)
        generate_direction 0 1 1 []         (* right *)
      ]

(* Try to make a move from current state *)
let try_move (current_state: state) (maze: maze) =
  let (curr_cell, movement, prev_steps) = current_state in
  let current_pos = prev_steps.(Array.length prev_steps - 1) in
  let prev_pos = if Array.length prev_steps > 1 then 
                   Some (prev_steps.(Array.length prev_steps - 2))
                 else None in
  
  let max_steps = 3 in
  
  (* First determine the next movement type based on current cell *)
  let next_movement = 
    if curr_cell.circled then
      (* If current cell is circled, flip the movement type *)
      if movement = Diagonal then Normal else Diagonal
    else
      (* If not circled, keep the same movement type *)
      movement
  in
  
  (* Generate positions based on the next movement type *)
  let possible_positions = generate_positions (fst current_pos) (snd current_pos) next_movement max_steps prev_pos in
  
  (* Filter valid moves and generate new states *)
  List.filter_map (fun pos ->
    match get_cell maze pos with
    | Some next_cell when 
        (next_cell.shape = curr_cell.shape || next_cell.color = curr_cell.color) &&
        not (pos_equal pos current_pos) &&
        (match prev_pos with
         | Some p -> 
            let next_dir = get_direction current_pos pos in
            let prev_dir = get_direction p current_pos in
            not (is_opposite_direction next_dir prev_dir) &&
            not (pos_equal pos p)
         | None -> true) ->
          let new_steps = Array.append prev_steps [|pos|] in
          Some (next_cell, next_movement, new_steps)
    | _ -> None
  ) possible_positions

(* Search for all possible paths up to n steps *)
let search_n_steps maze =
  let initial_state = match get_cell maze maze.start_point with
    | Some cell -> (cell, Diagonal, [|maze.start_point|])
    | None -> failwith "Invalid start position"
  in
  
  let end_cell = match get_cell maze maze.end_point with
    | Some cell -> cell
    | None -> failwith "Invalid end position"
  in

  let max_steps = 18 in
  let rec search_steps current_states step_count completed_paths =
    if step_count >= max_steps || List.length current_states = 0 then
      completed_paths
    else (
      let next_states = List.concat_map (fun state -> try_move state maze) current_states in
      let (new_completed, still_searching) = 
        List.partition 
          (fun (cell, _, steps) -> 
            let last_pos = steps.(Array.length steps - 1) in
            pos_equal last_pos maze.end_point && 
            (cell.shape = end_cell.shape || cell.color = end_cell.color))
          next_states
      in
      search_steps still_searching (step_count + 1) (completed_paths @ new_completed)
    )
  in
  
  let final_paths = search_steps [initial_state] 0 [] in
  
  print_endline "\nPaths that reached the end point:";
  List.iteri (fun i (cell, movement, steps) ->
    print_endline (Printf.sprintf "Len %d Path %d: %s" 
      (Array.length steps)
      (i + 1)
      (display_path maze steps))
  ) final_paths;
  print_endline (Printf.sprintf "Found %d paths to end point" (List.length final_paths))

(* Create the maze grid *)
let create_maze () =
  { 
    grid = [|
      [|None; None; Some { shape = Diamond; color = Green; circled = false }; None; None|];
      [|None; Some { shape = Star; color = Purple; circled = true }; Some { shape = Star; color = Blue; circled = true }; Some { shape = Star; color = Pink; circled = true }; None|];
      [|Some { shape = Diamond; color = Green; circled = false }; Some { shape = Diamond; color = Green; circled = true }; Some { shape = Diamond; color = Purple; circled = false }; Some { shape = Star; color = Green; circled = false }; Some { shape = Star; color = Green; circled = false }|];
      [|None; Some { shape = Diamond; color = Purple; circled = true }; Some { shape = Hexagon; color = Green; circled = true }; Some { shape = Diamond; color = Green; circled = false }; None|];
      [|None;  None; Some { shape = Triangle; color = Blue; circled = false }; None; None|]
    |];
    start_point = (0, 2);
    end_point = (4, 2)
  }

(* Main program *)
let () =
  let maze = create_maze () in
  print_endline "Maze structure:";
  display_maze maze;
  search_n_steps maze
