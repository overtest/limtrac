#ifndef DEMO_C_MAIN_H
#define DEMO_C_MAIN_H

#include "../../bindings/limtrac.h"

ExecProgInfo   get_exec_prog_info();
ExecProgIO     get_exec_prog_io();
ExecProgLimits get_exec_prog_limits();
ExecProgGuard  get_exec_prog_guard();

#endif //DEMO_C_MAIN_H