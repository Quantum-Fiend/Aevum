/*
 * Aevum C/C++ Agent - LLVM IR Instrumentation
 * 
 * This is a stub implementation showing the architecture for C/C++ tracing.
 * Full implementation requires LLVM pass development.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <pthread.h>
#include <stdint.h>
#include <time.h>

#define BUFFER_SIZE 4096

typedef struct {
    char trace_id[64];
    char server_host[256];
    int server_port;
    int socket_fd;
    int enabled;
    uint64_t sequence_number;
    pthread_mutex_t lock;
} AevumAgent;

static AevumAgent global_agent = {0};

// Initialize the agent
int aevum_agent_init(const char* trace_id, const char* server_host, int server_port) {
    strncpy(global_agent.trace_id, trace_id, sizeof(global_agent.trace_id) - 1);
    strncpy(global_agent.server_host, server_host, sizeof(global_agent.server_host) - 1);
    global_agent.server_port = server_port;
    global_agent.sequence_number = 0;
    pthread_mutex_init(&global_agent.lock, NULL);
    
    // Connect to coordinator
    global_agent.socket_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (global_agent.socket_fd < 0) {
        perror("Socket creation failed");
        return -1;
    }
    
    struct sockaddr_in server_addr;
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(server_port);
    inet_pton(AF_INET, server_host, &server_addr.sin_addr);
    
    if (connect(global_agent.socket_fd, (struct sockaddr*)&server_addr, sizeof(server_addr)) < 0) {
        perror("Connection failed");
        return -1;
    }
    
    global_agent.enabled = 1;
    printf("[Aevum] C/C++ agent initialized (trace_id: %s)\n", trace_id);
    return 0;
}

// Cleanup the agent
void aevum_agent_cleanup() {
    global_agent.enabled = 0;
    if (global_agent.socket_fd >= 0) {
        close(global_agent.socket_fd);
    }
    pthread_mutex_destroy(&global_agent.lock);
    printf("[Aevum] C/C++ agent stopped\n");
}

// Send event to coordinator
static void send_event(const char* event_json) {
    if (!global_agent.enabled) return;
    
    pthread_mutex_lock(&global_agent.lock);
    
    uint32_t length = strlen(event_json);
    uint32_t network_length = htonl(length);
    
    send(global_agent.socket_fd, &network_length, sizeof(network_length), 0);
    send(global_agent.socket_fd, event_json, length, 0);
    
    pthread_mutex_unlock(&global_agent.lock);
}

// Function entry hook (called by LLVM instrumentation)
void __aevum_function_entry(const char* function_name, const char* module) {
    if (!global_agent.enabled) return;
    
    pthread_mutex_lock(&global_agent.lock);
    uint64_t seq = ++global_agent.sequence_number;
    pthread_mutex_unlock(&global_agent.lock);
    
    char event_json[BUFFER_SIZE];
    snprintf(event_json, sizeof(event_json),
        "{\"event_type\":\"FunctionCall\","
        "\"metadata\":{\"trace_id\":\"%s\",\"process_id\":%d,\"thread_id\":%lu,"
        "\"timestamp_ns\":%lu,\"sequence_number\":%lu},"
        "\"function_name\":\"%s\",\"module\":\"%s\"}",
        global_agent.trace_id, getpid(), pthread_self(),
        (uint64_t)time(NULL) * 1000000000, seq,
        function_name, module);
    
    send_event(event_json);
}

// Function exit hook (called by LLVM instrumentation)
void __aevum_function_exit(const char* function_name) {
    if (!global_agent.enabled) return;
    
    pthread_mutex_lock(&global_agent.lock);
    uint64_t seq = ++global_agent.sequence_number;
    pthread_mutex_unlock(&global_agent.lock);
    
    char event_json[BUFFER_SIZE];
    snprintf(event_json, sizeof(event_json),
        "{\"event_type\":\"FunctionReturn\","
        "\"metadata\":{\"trace_id\":\"%s\",\"process_id\":%d,\"thread_id\":%lu,"
        "\"timestamp_ns\":%lu,\"sequence_number\":%lu},"
        "\"function_name\":\"%s\"}",
        global_agent.trace_id, getpid(), pthread_self(),
        (uint64_t)time(NULL) * 1000000000, seq,
        function_name);
    
    send_event(event_json);
}

// Memory write hook (called by LLVM instrumentation)
void __aevum_memory_write(void* address, size_t size) {
    if (!global_agent.enabled) return;
    
    pthread_mutex_lock(&global_agent.lock);
    uint64_t seq = ++global_agent.sequence_number;
    pthread_mutex_unlock(&global_agent.lock);
    
    char event_json[BUFFER_SIZE];
    snprintf(event_json, sizeof(event_json),
        "{\"event_type\":\"MemoryWrite\","
        "\"metadata\":{\"trace_id\":\"%s\",\"process_id\":%d,\"thread_id\":%lu,"
        "\"timestamp_ns\":%lu,\"sequence_number\":%lu},"
        "\"address\":%lu,\"size\":%zu}",
        global_agent.trace_id, getpid(), pthread_self(),
        (uint64_t)time(NULL) * 1000000000, seq,
        (uint64_t)address, size);
    
    send_event(event_json);
}

/*
 * LLVM Pass Implementation (Conceptual)
 * 
 * The LLVM pass would:
 * 1. Iterate through all functions in the module
 * 2. Insert calls to __aevum_function_entry at function entry
 * 3. Insert calls to __aevum_function_exit before returns
 * 4. Insert calls to __aevum_memory_write before store instructions
 * 
 * Build with:
 *   clang -Xclang -load -Xclang AevumPass.so program.c -o program
 */
