---- MODULE squidbleed_detection ----
EXTENDS Integers, Sequences

CONSTANTS MAX_BUFFER, MAX_STRING

VARIABLES
  buffer,        (* \in Seq(0..255) *)
  pointer,       (* 0..MAX_BUFFER *)
  copyFrom,      (* 0..MAX_BUFFER *)
  end_of_string  (* 0..MAX_BUFFER *)

(* Função strchr no padrão C11 — inclui o terminador nulo na busca *)
strchr(ch, str, i) ==
    LET pos == CHOOSE j \in i..Len(str)-1 : str[j] = ch
    IN IF pos = Len(str)-1 THEN pos ELSE pos

(* ============================================================ *)
(* O BUG QUE CAUSOU O SQUIDBLEED: strchr sem verificação de \0 *)
(* ============================================================ *)

(* Loop vulnerável — exatamente o que estava no Squid Proxy *)
VulnerableLoop ==
    /\ pointer < end_of_string
    /\ copyFrom < end_of_string
    /\ strchr(32, buffer, pointer) \in {pointer, pointer+1, ..., end_of_string}
    /\ pointer' = strchr(32, buffer, pointer)
    /\ UNCHANGED << buffer, copyFrom, end_of_string >>

(* == CORREÇÃO == *)
(* Loop correto: verifica se chegou ao fim da string antes de chamar strchr *)
CorrectLoop ==
    /\ pointer <= end_of_string
    /\ IF pointer < end_of_string /\ buffer[pointer] # 0
       THEN
           LET pos == strchr(32, buffer, pointer)
           IN pointer' = IF pos < end_of_string THEN pos + 1 ELSE pointer
       ELSE
           pointer' = pointer
    /\ UNCHANGED << buffer, copyFrom, end_of_string >>

(* Invariante de segurança: pointer nunca ultrapassa end_of_string *)
Invariant ==
    pointer <= end_of_string

(* Propriedade que deve ser verificada: o loop nunca ultrapassa o buffer *)
SafetyProperty ==
    [] (CorrectLoop => Invariant)

(* Propriedade violada pelo bug: *)
VulnerableProperty ==
    [] (VulnerableLoop => Invariant)  (* FALSO! *)

====
