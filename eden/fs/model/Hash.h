/*
 *  Copyright (c) 2016, Facebook, Inc.
 *  All rights reserved.
 *
 *  This source code is licensed under the BSD-style license found in the
 *  LICENSE file in the root directory of this source tree. An additional grant
 *  of patent rights can be found in the PATENTS file in the same directory.
 *
 */
#pragma once

#include <folly/Range.h>
#include <stdint.h>
#include <array>
#include <boost/operators.hpp>

namespace facebook {
namespace eden {

/**
 * Immutable 160-bit hash.
 */
class Hash : boost::totally_ordered<Hash> {
 public:
  enum { RAW_SIZE = 20 };

  explicit Hash(std::array<uint8_t, RAW_SIZE> bytes);

  explicit Hash(folly::ByteRange bytes);

  /**
   * @param hex is a string of 40 hexadecimal characters.
   */
  explicit Hash(folly::StringPiece hex);

  const std::array<uint8_t, RAW_SIZE>& getBytes() const;

  /** @return 40-character [lowercase] hex representation of this hash. */
  std::string toString() const;

  bool operator==(const Hash&) const;
  bool operator<(const Hash&) const;

 private:
  const std::array<uint8_t, RAW_SIZE> bytes_;
};
}
}
