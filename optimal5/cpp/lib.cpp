
#include <cstdint>
#include <stdint.h>
#include <string>

#include "optimal5.cpp"

// Global instance (unused but kept for future extensibility)
[[maybe_unused]] static opt5::Optimiser *global_opt = nullptr;

int calculate_depth(opt5::gatevec_fast &gv) {
  // TODO: revisar
  if (gv.n_gates == 0) {
    return 0; // Solo inputs, depth = 0
  }

  int depths[18] = {0};

  for (int idx = 0; idx < gv.n_gates; idx++) {
    uint16_t gate = gv.gates[idx];

    int i1 = (gate >> 4) & 31;
    int i2 = (gate >> 10) & 31;

    int gate_depth = 1 + std::max(depths[i1], depths[i2]);
    depths[idx + 6] = gate_depth; // Este gate es x(idx+6)
  }

  // La depth total es la depth del gate de salida
  int output_gate = gv.output >> 1;
  return depths[output_gate];
}

class WrapperOptimiser {
public:
  opt5::Optimiser *opt;
  WrapperOptimiser(std::vector<uint8_t> bv) { opt = new opt5::Optimiser(bv); }

  ~WrapperOptimiser() { delete opt; }
};

extern "C" {

void *create_wrapper(uint8_t *bv) {
  std::vector<uint8_t> bv_vec(bv, bv + 9241860); // Assuming size of bv is 100
  auto wrapper = new WrapperOptimiser(bv_vec);
  return reinterpret_cast<void *>(wrapper);
}

uint8_t num_gates(void *wrapper, uint32_t fun) {
  auto gv = reinterpret_cast<WrapperOptimiser *>(wrapper)->opt->lookup(fun);
  return gv.n_gates;
}

uint32_t npn_representant(void *wrapper, uint32_t fun) {
  return reinterpret_cast<WrapperOptimiser *>(wrapper)
      ->opt->lookup_representant(fun);
}

uint8_t calculate_depth(void *wrapper, uint32_t fun) {
  auto gv = reinterpret_cast<WrapperOptimiser *>(wrapper)->opt->lookup(fun);
  return calculate_depth(gv);
}
}
