/* See LICENSE file for copyright and license details. */
#include <stddef.h>

static char *const rcinitcmd[]     = { "/bin/rc.init", NULL };
static char *const rcrebootcmd[]   = { "/bin/rc.shutdown", "reboot", NULL };
static char *const rcpoweroffcmd[] = { "/bin/rc.shutdown", "poweroff", NULL };
