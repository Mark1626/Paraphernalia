------------------------------ MODULE diehard ------------------------------

EXTENDS Integers, Sequences, TLC, FiniteSets

Min(m, n) == IF m < n THEN m ELSE n

(* --algorithm diehard
variables
    big = 0;
    small = 0;
define
    TypeInvariant ==
        /\ big \in 0..5
        /\ small \in 0..3
end define
begin
    Iterate:
        while big # 4 do
            either big := 5
            or     small := 3
            or     big := 0
            or     small := 0
            or
                with poured = Min(big + small, 5) - big do
                    big   := big + poured;
                    small := small - poured;
                end with;
            or
                with poured = Min(big + small, 3) - small do
                    big := big - poured;
                    small := small + poured;
                end with;
            end either;
        end while;
end algorithm; *)
\* BEGIN TRANSLATION (chksum(pcal) = "1f43ca45" /\ chksum(tla) = "f9b32565")
VARIABLES big, small, pc

(* define statement *)
TypeInvariant ==
    /\ big \in 0..5
    /\ small \in 0..3


vars == << big, small, pc >>

Init == (* Global variables *)
        /\ big = 0
        /\ small = 0
        /\ pc = "Iterate"

Iterate == /\ pc = "Iterate"
           /\ IF big # 4
                 THEN /\ \/ /\ big' = 5
                            /\ small' = small
                         \/ /\ small' = 3
                            /\ big' = big
                         \/ /\ big' = 0
                            /\ small' = small
                         \/ /\ small' = 0
                            /\ big' = big
                         \/ /\ LET poured == Min(big + small, 5) - big IN
                                 /\ big' = big + poured
                                 /\ small' = small - poured
                         \/ /\ LET poured == Min(big + small, 3) - small IN
                                 /\ big' = big - poured
                                 /\ small' = small + poured
                      /\ pc' = "Iterate"
                 ELSE /\ pc' = "Done"
                      /\ UNCHANGED << big, small >>

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == pc = "Done" /\ UNCHANGED vars

Next == Iterate
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(pc = "Done")

\* END TRANSLATION 

=============================================================================
\* Modification History
\* Last modified Sun Dec 28 09:25:38 IST 2025 by nimalanm
\* Created Sat Dec 27 20:12:42 IST 2025 by nimalanm
