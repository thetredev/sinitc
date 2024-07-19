/* See LICENSE file for copyright and license details. */
#include <sys/types.h>
#include <sys/wait.h>

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#ifndef __USE_POSIX
#define __USE_POSIX
#endif // __USE_POSIX

#include <signal.h>

#define LEN(x)    (sizeof (x) / sizeof *(x))
#define TIMEOUT   30

static void sigpoweroff(void);
static void sigreap(void);
static void spawn(char *const []);

static struct {
    int sig;
    void (*handler)(void);
} sigmap[] = {
    { SIGUSR1, sigpoweroff },
    { SIGINT,  sigpoweroff },
    { SIGKILL, sigpoweroff },
    { SIGTERM, sigpoweroff },
    { SIGCHLD, sigreap     },
    { SIGALRM, sigreap     },
};


#include "config.h"

static sigset_t set;

int main(void) {
    if (getpid() != 1) {
        return 1;
    }

    chdir("/");
    sigfillset(&set);
    sigprocmask(SIG_BLOCK, &set, NULL);
    spawn(rcinitcmd);

    int sig;
    size_t i;

    while (1) {
        alarm(TIMEOUT);
        sigwait(&set, &sig);

        for (i = 0; i < LEN(sigmap); i++) {
            if (sigmap[i].sig == sig) {
                sigmap[i].handler();
                break;
            }
        }
    }

    /* not reachable */
    return 0;
}

static void sigpoweroff(void) {
    spawn(rcpoweroffcmd);
}

static void sigreap(void) {
    while (waitpid(-1, NULL, WNOHANG) > 0);
    alarm(TIMEOUT);
}

static void spawn(char *const argv[]) {
    switch (fork()) {
    case 0:
        sigprocmask(SIG_UNBLOCK, &set, NULL);
        setsid();
        execvp(argv[0], argv);

        perror("execvp");
        _exit(1);
    case -1:
        perror("fork");
    }
}
