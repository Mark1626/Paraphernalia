------------------------------- MODULE queue -------------------------------

EXTENDS TLC, Integers, Sequences

CONSTANT Workers
CONSTANT Tasks

(*
--algorithm queue {

variables
    queue   = <<>>;
    Readers = 1..Workers;
    done    = [i \in 1..Tasks |-> FALSE]; 

process (Writer = 0)
variable j = 0;
{
    WS:             await queue = <<>>;

                    j := 1;
    W1:             while (j <= Tasks) {
                        queue := Append(queue, j);
                        j := j + 1;
                    };

    W2:             await \A idx \in 1..Tasks : done[idx] = TRUE;
                    assert queue = <<>>;
}

process (Reader \in Readers)
variable work = 0;
{
    RS:             await queue # <<>>;
                    work  := Head(queue);
                    queue := Tail(queue);
    
    R0:             done[work] := TRUE;
                    goto RS;
}

} \* end algorithm
*)
\* BEGIN TRANSLATION (chksum(pcal) = "c6f7e699" /\ chksum(tla) = "2c471d62")
VARIABLES queue, Readers, done, pc, j, work

vars == << queue, Readers, done, pc, j, work >>

ProcSet == {0} \cup (Readers)

Init == (* Global variables *)
        /\ queue = <<>>
        /\ Readers = 1..Workers
        /\ done = [i \in 1..Tasks |-> FALSE]
        (* Process Writer *)
        /\ j = 0
        (* Process Reader *)
        /\ work = [self \in Readers |-> 0]
        /\ pc = [self \in ProcSet |-> CASE self = 0 -> "WS"
                                        [] self \in Readers -> "RS"]

WS == /\ pc[0] = "WS"
      /\ queue = <<>>
      /\ j' = 1
      /\ pc' = [pc EXCEPT ![0] = "W1"]
      /\ UNCHANGED << queue, Readers, done, work >>

W1 == /\ pc[0] = "W1"
      /\ IF j <= Tasks
            THEN /\ queue' = Append(queue, j)
                 /\ j' = j + 1
                 /\ pc' = [pc EXCEPT ![0] = "W1"]
            ELSE /\ pc' = [pc EXCEPT ![0] = "W2"]
                 /\ UNCHANGED << queue, j >>
      /\ UNCHANGED << Readers, done, work >>

W2 == /\ pc[0] = "W2"
      /\ \A idx \in 1..Tasks : done[idx] = TRUE
      /\ Assert(queue = <<>>, "Failure of assertion at line 28, column 21.")
      /\ pc' = [pc EXCEPT ![0] = "Done"]
      /\ UNCHANGED << queue, Readers, done, j, work >>

Writer == WS \/ W1 \/ W2

RS(self) == /\ pc[self] = "RS"
            /\ queue # <<>>
            /\ work' = [work EXCEPT ![self] = Head(queue)]
            /\ queue' = Tail(queue)
            /\ pc' = [pc EXCEPT ![self] = "R0"]
            /\ UNCHANGED << Readers, done, j >>

R0(self) == /\ pc[self] = "R0"
            /\ done' = [done EXCEPT ![work[self]] = TRUE]
            /\ pc' = [pc EXCEPT ![self] = "RS"]
            /\ UNCHANGED << queue, Readers, j, work >>

Reader(self) == RS(self) \/ R0(self)

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == Writer
           \/ (\E self \in Readers: Reader(self))
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 

=============================================================================
\* Modification History
\* Last modified Fri Jan 02 12:36:06 IST 2026 by nimalanm
\* Created Fri Jan 02 12:17:29 IST 2026 by nimalanm
