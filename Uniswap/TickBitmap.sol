// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {BitMath} from './BitMath.sol';

/// @title Packed tick initialized state library
/// @notice Stores a packed mapping of tick index to its initialized state
/// @dev The mapping uses int16 for keys since ticks are represented as int24 and there are 256 (2^8) values per word.
library TickBitmap {
    /// @notice Computes the position in the mapping where the initialized bit for a tick lives
    /// @param tick The tick for which to compute the position
    /// @return wordPos The key in the mapping containing the word in which the bit is stored
    /// @return bitPos The bit position in the word where the flag is stored
    function position(int24 tick) private pure returns (int16 wordPos, uint8 bitPos) {
        unchecked {
            wordPos = int16(tick >> 8);
            bitPos = uint8(int8(tick % 256));
        }
    }

    /// @notice Flips the initialized state for a given tick from false to true, or vice versa
    /// @param self The mapping in which to flip the tick
    /// @param tick The tick to flip
    /// @param tickSpacing The spacing between usable ticks
    function flipTick(
        mapping(int16 => uint256) storage self,
        int24 tick,
        int24 tickSpacing
    ) internal {
        unchecked {
            require(tick % tickSpacing == 0); // ensure that the tick is spaced
            (int16 wordPos, uint8 bitPos) = position(tick / tickSpacing);
            uint256 mask = 1 << bitPos;
            self[wordPos] ^= mask;
        }
    }

    /// @notice Returns the next initialized tick contained in the same word (or adjacent word) as the tick that is either
    /// to the left (less than or equal to) or right (greater than) of the given tick
    /// @param self The mapping in which to compute the next initialized tick
    /// @param tick The starting tick
    /// @param tickSpacing The spacing between usable ticks
    /// @param lte Whether to search for the next initialized tick to the left (less than or equal to the starting tick)
    /// @return next The next initialized or uninitialized tick up to 256 ticks away from the current tick
    /// @return initialized Whether the next tick is initialized, as the function only searches within up to 256 ticks
    function nextInitializedTickWithinOneWord(
        mapping(int16 => uint256) storage self,
        int24 tick,
        int24 tickSpacing,
        bool lte
    ) internal view returns (int24 next, bool initialized) {
        unchecked {
            int24 compressed = tick / tickSpacing;
            if (tick < 0 && tick % tickSpacing != 0) compressed--; // round towards negative infinity

            if (lte) {
                (int16 wordPos, uint8 bitPos) = position(compressed);
                // all the 1s at or to the right of the current bitPos
                uint256 mask = (1 << bitPos) - 1 + (1 << bitPos);
                uint256 masked = self[wordPos] & mask;

                // if there are no initialized ticks to the right of or at the current tick, return rightmost in the word
                initialized = masked != 0;
                // overflow/underflow is possible, but prevented externally by limiting both tickSpacing and tick
                next = initialized
                    ? (compressed - int24(uint24(bitPos - BitMath.mostSignificantBit(masked)))) * tickSpacing
                    : (compressed - int24(uint24(bitPos))) * tickSpacing;
            } else {
                // start from the word of the next tick, since the current tick state doesn't matter
                (int16 wordPos, uint8 bitPos) = position(compressed + 1);
                // all the 1s at or to the left of the bitPos
                uint256 mask = ~((1 << bitPos) - 1);
                uint256 masked = self[wordPos] & mask;

                // if there are no initialized ticks to the left of the current tick, return leftmost in the word
                initialized = masked != 0;
                // overflow/underflow is possible, but prevented externally by limiting both tickSpacing and tick
                next = initialized
                    ? (compressed + 1 + int24(uint24(BitMath.leastSignificantBit(masked) - bitPos))) * tickSpacing
                    : (compressed + 1 + int24(uint24(type(uint8).max - bitPos))) * tickSpacing;
            }
        }
    }
}it('returns tick to right if at initialized tick', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(78, false)
//         expect(next).to.eq(84)
//         expect(initialized).to.eq(true)
//       })
//       it('returns tick to right if at initialized tick', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(-55, false)
//         expect(next).to.eq(-4)
//         expect(initialized).to.eq(true)
//       })

//       it('returns the tick directly to the right', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(77, false)
//         expect(next).to.eq(78)
//         expect(initialized).to.eq(true)
//       })
//       it('returns the tick directly to the right', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(-56, false)
//         expect(next).to.eq(-55)
//         expect(initialized).to.eq(true)
//       })

//       it('returns the next words initialized tick if on the right boundary', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(255, false)
//         expect(next).to.eq(511)
//         expect(initialized).to.eq(false)
//       })
//       it('returns the next words initialized tick if on the right boundary', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(-257, false)
//         expect(next).to.eq(-200)
//         expect(initialized).to.eq(true)
//       })

//       it('returns the next initialized tick from the next word', async () => {
//         await tickBitmap.flipTick(340)
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(328, false)
//         expect(next).to.eq(340)
//         expect(initialized).to.eq(true)
//       })
//       it('does not exceed boundary', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(508, false)
//         expect(next).to.eq(511)
//         expect(initialized).to.eq(false)
//       })
//       it('skips entire word', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(255, false)
//         expect(next).to.eq(511)
//         expect(initialized).to.eq(false)
//       })
//       it('skips half word', async () => {
//         const { next, initialized } = await tickBitmap.nextInitializedTickWithinOneWord(383, false)
//         expect(next).to.eq(511)
//         expect(initialized).to.eq(false)
//       })