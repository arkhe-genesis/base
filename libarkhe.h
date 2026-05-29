// libarkhe.h — API pública para processos userspace
#ifndef ARKHE_H
#define ARKHE_H

#include <stdint.h>
#include <stddef.h>

// Inicializa a conexão com o módulo de kernel
int arkhe_init(void);

// Sela um arquivo (calcula hash e o ancora na TemporalChain)
int arkhe_seal(const char *path, char *seal_hash, size_t hash_len);

// Persiste um estado epistêmico (L0-L5)
int arkhe_commit(const char *intent, const void *state, size_t state_len);

// Recupera histórico de eventos para um arquivo
int arkhe_audit(const char *path, char *events_json, size_t json_len);

#endif // ARKHE_H
