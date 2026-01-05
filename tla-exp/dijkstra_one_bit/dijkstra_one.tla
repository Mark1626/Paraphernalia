---------------------------- MODULE dijkstra_one ----------------------------

EXTENDS TLC, Naturals

(*  
    Dijstra one bit / fast mutex algorithm from 
    https://lamport.azurewebsites.net/pubs/dijkstra.pdf
*)

CONSTANT N

(*
--algorithm dijkstra_fastmutex {
  variables flag = [i \in 1..N |-> FALSE ];
            x = 0,
            y = 0;
  process (thread \in 1..N)
    variable j = 0;
    {
        NonCritical: skip;
        
        \* Synchronization
        start:       flag[self] := TRUE;
        S01:         x := self;
        
        S02:        if ( y # 0 ) {
        S03:            flag[self] := FALSE;
        S04:            await y = 0;
                        goto start;
                    };
        
        S05:        y := self;
        S06:        if (x # self) {
        S07:            flag[self] := FALSE;
                        j := 1;
                        \* for j in 1..N await ~flag[j]
        S08:            while (j <= N) {
                            await ~flag[j];
                            j := j + 1;
                        };

        S09:            if (y # self) {
        S10:                await y = 0;
                            goto start;
                        };
                    };

        CriticalSection: skip;
                         assert \A idx \in 1..N : (idx # self) => (pc[idx] # "CriticalSection");
        S11:             y := 0;
        S12:             flag[self] := FALSE; \* Set the flag back to FALSE AS we exit the critical section
                         
                         goto NonCritical;
      } \*  end while;
    } \*end process 
} \*end algorithm;
*)
\* BEGIN TRANSLATION (chksum(pcal) = "37ef2b1a" /\ chksum(tla) = "f4bc4f7a")
VARIABLES flag, x, y, pc, j

vars == << flag, x, y, pc, j >>

ProcSet == (1..N)

Init == (* Global variables *)
        /\ flag = [i \in 1..N |-> FALSE ]
        /\ x = 0
        /\ y = 0
        (* Process thread *)
        /\ j = [self \in 1..N |-> 0]
        /\ pc = [self \in ProcSet |-> "NonCritical"]

NonCritical(self) == /\ pc[self] = "NonCritical"
                     /\ TRUE
                     /\ pc' = [pc EXCEPT ![self] = "start"]
                     /\ UNCHANGED << flag, x, y, j >>

start(self) == /\ pc[self] = "start"
               /\ flag' = [flag EXCEPT ![self] = TRUE]
               /\ pc' = [pc EXCEPT ![self] = "S01"]
               /\ UNCHANGED << x, y, j >>

S01(self) == /\ pc[self] = "S01"
             /\ x' = self
             /\ pc' = [pc EXCEPT ![self] = "S02"]
             /\ UNCHANGED << flag, y, j >>

S02(self) == /\ pc[self] = "S02"
             /\ IF y # 0
                   THEN /\ pc' = [pc EXCEPT ![self] = "S03"]
                   ELSE /\ pc' = [pc EXCEPT ![self] = "S05"]
             /\ UNCHANGED << flag, x, y, j >>

S03(self) == /\ pc[self] = "S03"
             /\ flag' = [flag EXCEPT ![self] = FALSE]
             /\ pc' = [pc EXCEPT ![self] = "S04"]
             /\ UNCHANGED << x, y, j >>

S04(self) == /\ pc[self] = "S04"
             /\ y = 0
             /\ pc' = [pc EXCEPT ![self] = "start"]
             /\ UNCHANGED << flag, x, y, j >>

S05(self) == /\ pc[self] = "S05"
             /\ y' = self
             /\ pc' = [pc EXCEPT ![self] = "S06"]
             /\ UNCHANGED << flag, x, j >>

S06(self) == /\ pc[self] = "S06"
             /\ IF x # self
                   THEN /\ pc' = [pc EXCEPT ![self] = "S07"]
                   ELSE /\ pc' = [pc EXCEPT ![self] = "CriticalSection"]
             /\ UNCHANGED << flag, x, y, j >>

S07(self) == /\ pc[self] = "S07"
             /\ flag' = [flag EXCEPT ![self] = FALSE]
             /\ j' = [j EXCEPT ![self] = 1]
             /\ pc' = [pc EXCEPT ![self] = "S08"]
             /\ UNCHANGED << x, y >>

S08(self) == /\ pc[self] = "S08"
             /\ IF j[self] <= N
                   THEN /\ ~flag[j[self]]
                        /\ j' = [j EXCEPT ![self] = j[self] + 1]
                        /\ pc' = [pc EXCEPT ![self] = "S08"]
                   ELSE /\ pc' = [pc EXCEPT ![self] = "S09"]
                        /\ j' = j
             /\ UNCHANGED << flag, x, y >>

S09(self) == /\ pc[self] = "S09"
             /\ IF y # self
                   THEN /\ pc' = [pc EXCEPT ![self] = "S10"]
                   ELSE /\ pc' = [pc EXCEPT ![self] = "CriticalSection"]
             /\ UNCHANGED << flag, x, y, j >>

S10(self) == /\ pc[self] = "S10"
             /\ y = 0
             /\ pc' = [pc EXCEPT ![self] = "start"]
             /\ UNCHANGED << flag, x, y, j >>

CriticalSection(self) == /\ pc[self] = "CriticalSection"
                         /\ TRUE
                         /\ Assert(\A idx \in 1..N : (idx # self) => (pc[idx] # "CriticalSection"), 
                                   "Failure of assertion at line 49, column 26.")
                         /\ pc' = [pc EXCEPT ![self] = "S11"]
                         /\ UNCHANGED << flag, x, y, j >>

S11(self) == /\ pc[self] = "S11"
             /\ y' = 0
             /\ pc' = [pc EXCEPT ![self] = "S12"]
             /\ UNCHANGED << flag, x, j >>

S12(self) == /\ pc[self] = "S12"
             /\ flag' = [flag EXCEPT ![self] = FALSE]
             /\ pc' = [pc EXCEPT ![self] = "NonCritical"]
             /\ UNCHANGED << x, y, j >>

thread(self) == NonCritical(self) \/ start(self) \/ S01(self) \/ S02(self)
                   \/ S03(self) \/ S04(self) \/ S05(self) \/ S06(self)
                   \/ S07(self) \/ S08(self) \/ S09(self) \/ S10(self)
                   \/ CriticalSection(self) \/ S11(self) \/ S12(self)

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == (\E self \in 1..N: thread(self))
           \/ Terminating

Spec == Init /\ [][Next]_vars

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION 

=============================================================================
\* Modification History
\* Last modified Tue Dec 30 13:36:00 IST 2025 by nimalanm
\* Created Tue Dec 30 10:25:14 IST 2025 by nimalanm
