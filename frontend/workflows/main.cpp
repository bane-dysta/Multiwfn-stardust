#include "banewfn/application.h"
#include "common/input.h"

#include <iostream>
#include <exception>
#include <string>
#include <vector>

// This desktop sidecar always uses batch stdin. Scientific work stays in Multiwfn.
int main(int argc, char* argv[]) {
    if (argc > 1 && argv[1][0] != '-') {
        const auto input = InputParser::parseInpFileDetailed(argv[1]);
        if (!input.loaded) return 1;
        if (input.debug) {
            std::cerr << "Error: debug mode requires a terminal; disable it for desktop workflows.\n";
            return 2;
        }
        for (const auto& task : input.tasks) {
            if (task.useWait) {
                std::cerr << "Error: wait requires a terminal; use end for desktop workflows.\n";
                return 2;
            }
        }
    }
    std::vector<char*> args(argv, argv + argc);
    std::string nogui = "--nogui";
    args.push_back(nogui.data());
    args.push_back(nullptr);
    try {
        return BaneWfn::run(static_cast<int>(args.size() - 1), args.data());
    } catch (const std::exception& error) {
        std::cerr << "Error: " << error.what() << '\n';
        return 2;
    }
}
