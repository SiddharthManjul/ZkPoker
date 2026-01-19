/**
 * Test proof generation for all circuits in isolation
 * This ensures the prover utilities work correctly before integration tests
 */

import { describe, it } from "mocha";
import { expect } from "chai";
import * as crypto from "crypto";
import { keccak256 } from "js-sha3";

import {
  shuffleDeck,
  getHoleCards,
  getFlopCards,
  commitDeck,
  verifyDeckIntegrity,
} from "./utils/deck";
import { generateSalt } from "./utils/crypto";
import { generateDeckProof, generateRevealProof, generateShowdownProof, verifyProofSize } from "./utils/prover";
import { evaluateHandFromIndices, createSevenCardHand } from "./utils/hand-eval";

describe("ZK Proof Generation Tests", () => {
  let seed1: Buffer;
  let seed2: Buffer;
  let deckSeed: Buffer;
  let shuffledDeck: number[];
  let player0Salt1: bigint, player0Salt2: bigint;
  let player0Commitments: [bigint, bigint];

  before("Setup test data", () => {
    // Generate seeds
    seed1 = crypto.randomBytes(32);
    seed2 = crypto.randomBytes(32);

    // Compute deck seed
    const combinedSeed = Buffer.concat([seed1, seed2]);
    deckSeed = Buffer.from(keccak256(combinedSeed), "hex");

    // Shuffle deck
    shuffledDeck = shuffleDeck(deckSeed);

    // Verify deck integrity
    expect(verifyDeckIntegrity(shuffledDeck)).to.be.true;

    console.log("✅ Test setup complete:");
    console.log(`  Deck seed: ${deckSeed.toString("hex").substring(0, 16)}...`);
    console.log(`  First 10 cards: [${shuffledDeck.slice(0, 10).join(", ")}]`);
  });

  describe("DECK Circuit Proof Generation", () => {
    it("should generate valid proof for player 0 hole cards", async function () {
      this.timeout(30000); // 30 seconds for proof generation

      const [card1, card2] = getHoleCards(shuffledDeck, 0);
      player0Salt1 = generateSalt();
      player0Salt2 = generateSalt();

      console.log(`  Player 0 cards: ${card1}, ${card2}`);

      const result = await generateDeckProof({
        deckSeed,
        playerSeat: 0,
        card1,
        card2,
        salt1: player0Salt1,
        salt2: player0Salt2,
      });

      player0Commitments = result.commitments;
      console.log(`  Commitments: ${result.commitments[0]}, ${result.commitments[1]}`);

      expect(result.proof).to.be.instanceOf(Buffer);
      expect(verifyProofSize(result.proof)).to.be.true;
      console.log(`  ✅ Proof generated: ${result.proof.length} bytes`);
    });

    it("should generate valid proof for player 1 hole cards", async function () {
      this.timeout(30000);

      const [card1, card2] = getHoleCards(shuffledDeck, 1);
      const salt1 = generateSalt();
      const salt2 = generateSalt();

      console.log(`  Player 1 cards: ${card1}, ${card2}`);

      const result = await generateDeckProof({
        deckSeed,
        playerSeat: 1,
        card1,
        card2,
        salt1,
        salt2,
      });

      console.log(`  Commitments: ${result.commitments[0]}, ${result.commitments[1]}`);

      expect(result.proof).to.be.instanceOf(Buffer);
      expect(verifyProofSize(result.proof)).to.be.true;
      console.log(`  ✅ Proof generated: ${result.proof.length} bytes`);
    });
  });

  describe("REVEAL Circuit Proof Generation", () => {
    it("should generate valid proof for flop reveal", async function () {
      this.timeout(30000);

      const flopCards = getFlopCards(shuffledDeck);
      console.log(`  Flop cards: [${flopCards.join(", ")}]`);

      const proof = await generateRevealProof({
        deckSeed,
        cards: Array.from(flopCards),
        numCards: 3,
        shuffledDeck,
      });

      expect(proof).to.be.instanceOf(Buffer);
      expect(verifyProofSize(proof)).to.be.true;
      console.log(`  ✅ Proof generated: ${proof.length} bytes`);
    });

    it("should generate valid proof for turn reveal", async function () {
      this.timeout(30000);

      const flopCards = getFlopCards(shuffledDeck);
      const turnCard = shuffledDeck[21];
      const cards = [...flopCards, turnCard];

      console.log(`  Turn card: ${turnCard}`);

      const proof = await generateRevealProof({
        deckSeed,
        cards,
        numCards: 4,
        shuffledDeck,
      });

      expect(proof).to.be.instanceOf(Buffer);
      expect(verifyProofSize(proof)).to.be.true;
      console.log(`  ✅ Proof generated: ${proof.length} bytes`);
    });

    it("should generate valid proof for river reveal", async function () {
      this.timeout(30000);

      const flopCards = getFlopCards(shuffledDeck);
      const turnCard = shuffledDeck[21];
      const riverCard = shuffledDeck[22];
      const cards = [...flopCards, turnCard, riverCard];

      console.log(`  River card: ${riverCard}`);

      const proof = await generateRevealProof({
        deckSeed,
        cards,
        numCards: 5,
        shuffledDeck,
      });

      expect(proof).to.be.instanceOf(Buffer);
      expect(verifyProofSize(proof)).to.be.true;
      console.log(`  ✅ Proof generated: ${proof.length} bytes`);
    });
  });

  describe("SHOWDOWN Circuit Proof Generation", () => {
    it("should generate valid proof for hand reveal and evaluation", async function () {
      this.timeout(30000);

      // Get player 0's hole cards (same as in DECK test)
      const [card1, card2] = getHoleCards(shuffledDeck, 0);

      // Get community cards
      const flopCards = getFlopCards(shuffledDeck);
      const turnCard = shuffledDeck[21];
      const riverCard = shuffledDeck[22];
      const communityCards: [number, number, number, number, number] = [
        ...flopCards,
        turnCard,
        riverCard,
      ];

      // Evaluate hand
      const sevenCards = createSevenCardHand([card1, card2], communityCards);
      const handRank = evaluateHandFromIndices(sevenCards);

      console.log(`  Hole cards: ${card1}, ${card2}`);
      console.log(`  Community: [${communityCards.join(", ")}]`);
      console.log(`  Hand rank: ${handRank}`);

      // Use commitments from DECK circuit, not TypeScript computation
      const proof = await generateShowdownProof({
        commitment1: player0Commitments[0],
        commitment2: player0Commitments[1],
        communityCards,
        holeCard1: card1,
        holeCard2: card2,
        salt1: player0Salt1,
        salt2: player0Salt2,
      });

      expect(proof).to.be.instanceOf(Buffer);
      expect(verifyProofSize(proof)).to.be.true;
      console.log(`  ✅ Proof generated: ${proof.length} bytes`);
    });
  });

  describe("Deck Shuffle Determinism", () => {
    it("should produce same deck from same seed", () => {
      const deck1 = shuffleDeck(deckSeed);
      const deck2 = shuffleDeck(deckSeed);

      expect(deck1).to.deep.equal(deck2);
      console.log("  ✅ Shuffle is deterministic");
    });

    it("should produce different decks from different seeds", () => {
      const otherSeed = crypto.randomBytes(32);
      const deck1 = shuffleDeck(deckSeed);
      const deck2 = shuffleDeck(otherSeed);

      expect(deck1).to.not.deep.equal(deck2);
      console.log("  ✅ Different seeds produce different shuffles");
    });
  });
});
