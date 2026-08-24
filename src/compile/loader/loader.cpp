#include <cstdio>
#include <string>
#include <fstream>
#include <stdexcept>

std::string load_in(const std::string& path) {
    std::ifstream file(path, std::ios::binary);

    if (!file) {
        throw std::runtime_error("File not found " + path);
    }

    file.seekg(0, std::ios::end);
    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::string contents;
    contents.resize(size);

    file.read(contents.data(), size);

    return contents;
}

std::ofstream load_out(const std::string& path) {
    std::string out_path = path;

    const size_t pos = out_path.find_last_of(".");

    if (pos != std::string::npos) {
        out_path.replace(pos, std::string::npos, ".out");
    } else {
        out_path += ".out";
    }

    std::ofstream out(
        out_path,
        std::ios::binary
    );

    if (!out) {
        throw std::runtime_error(
            "Could not open output file: " + out_path
        );
    }

    return out;
}
