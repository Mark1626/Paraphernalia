---------------------------- MODULE dining_phil ----------------------------

EXTENDS Naturals

CONSTANT N

ASSUME N \in Nat

Procs == 0..N-1

(*
--algorithm dining {

variables
    fork = [k \in Procs |-> N];
    
define {
    forkAvailable(i) == fork[i] = N
    LeftF(i)         == IF (i=N-1) THEN 0     ELSE i+1
    RightF(i)        == IF (i=0)   THEN (N-1) ELSE i-1     
}
    
process (j \in Procs)
variable state = "Think";
{
    L: while (TRUE) {
        L0: either {
            if (state = "Think") state := "Hungry";
        }
            
        or L1: {
            if (state = "Hungry") {
                await forkAvailable(RightF(self));
                fork[RightF(self)] := self;
            
            L2: await forkAvailable(LeftF(self));
                fork[LeftF(self)] := self;
            }
        }
            
        or L3: {
            if (state = "Eating") {
                state := "Thinking";
                fork[LeftF(self)] := N;
                
                L4: fork[RightF(self)] := N;
            }
        }
    };
}

}

*)
\* BEGIN TRANSLATION (chksum(pcal) = "94dfd979" /\ chksum(tla) = "1b1ea4dd")
VARIABLES fork, pc

(* define statement *)
forkAvailable(i) == fork[i] = N
LeftF(i)         == IF (i=N-1) THEN 0     ELSE i+1
RightF(i)        == IF (i=0)   THEN (N-1) ELSE i-1

VARIABLE state

vars == << fork, pc, state >>

ProcSet == (Procs)

Init == (* Global variables *)
        /\ fork = [k \in Procs |-> N]
        (* Process j *)
        /\ state = [self \in Procs |-> "Think"]
        /\ pc = [self \in ProcSet |-> "L"]

L(self) == /\ pc[self] = "L"
           /\ pc' = [pc EXCEPT ![self] = "L0"]
           /\ UNCHANGED << fork, state >>

L0(self) == /\ pc[self] = "L0"
            /\ \/ /\ IF state[self] = "Think"
                        THEN /\ state' = [state EXCEPT ![self] = "Hungry"]
                        ELSE /\ TRUE
                             /\ state' = state
                  /\ pc' = [pc EXCEPT ![self] = "L"]
               \/ /\ pc' = [pc EXCEPT ![self] = "L1"]
                  /\ state' = state
               \/ /\ pc' = [pc EXCEPT ![self] = "L3"]
                  /\ state' = state
            /\ fork' = fork

L1(self) == /\ pc[self] = "L1"
            /\ IF state[self] = "Hungry"
                  THEN /\ forkAvailable(RightF(self))
                       /\ fork' = [fork EXCEPT ![RightF(self)] = self]
                       /\ pc' = [pc EXCEPT ![self] = "L2"]
                  ELSE /\ pc' = [pc EXCEPT ![self] = "L"]
                       /\ fork' = fork
            /\ state' = state

L2(self) == /\ pc[self] = "L2"
            /\ forkAvailable(LeftF(self))
            /\ fork' = [fork EXCEPT ![LeftF(self)] = self]
            /\ pc' = [pc EXCEPT ![self] = "L"]
            /\ state' = state

L3(self) == /\ pc[self] = "L3"
            /\ IF state[self] = "Eating"
                  THEN /\ state' = [state EXCEPT ![self] = "Thinking"]
                       /\ fork' = [fork EXCEPT ![LeftF(self)] = N]
                       /\ pc' = [pc EXCEPT ![self] = "L4"]
                  ELSE /\ pc' = [pc EXCEPT ![self] = "L"]
                       /\ UNCHANGED << fork, state >>

L4(self) == /\ pc[self] = "L4"
            /\ fork' = [fork EXCEPT ![RightF(self)] = N]
            /\ pc' = [pc EXCEPT ![self] = "L"]
            /\ state' = state

j(self) == L(self) \/ L0(self) \/ L1(self) \/ L2(self) \/ L3(self)
              \/ L4(self)

Next == (\E self \in Procs: j(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION 


=============================================================================
\* Modification History
\* Last modified Mon Jan 05 16:35:06 IST 2026 by nimalanm
\* Created Mon Jan 05 16:26:21 IST 2026 by nimalanm
