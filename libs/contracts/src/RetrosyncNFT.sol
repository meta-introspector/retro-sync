// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/common/ERC2981.sol";

/**
 * @title RetrosyncNFT
 * @dev Enterprise-grade NFT representing a music release with Master Pattern metadata.
 * No artist names are stored; only wallet addresses and cryptographic hashes.
 */
contract RetrosyncNFT is ERC721URIStorage, ERC2981, Ownable {
    uint256 private _nextTokenId;

    struct Metadata {
        string isrc;
        string cid;
        uint8 band;
        uint256 releaseDate;
        string genre;
    }

    mapping(uint256 => Metadata) public tokenMetadata;
    mapping(uint256 => address) public creatorOf;

    event Minted(uint256 indexed tokenId, address indexed creator, string isrc, string cid);

    constructor(address initialOwner) 
        ERC721("Retrosync Release", "RSYNC") 
        Ownable(initialOwner) 
    {}

    /**
     * @dev Mints a new music release NFT.
     * @param artist The wallet address of the creator (no name stored).
     * @param tokenURI Metadata JSON URI.
     * @param isrc International Standard Recording Code.
     * @param cid BTFS Content Identifier.
     * @param band Master Pattern Band.
     * @param genre Genre from Wikidata enrichment.
     */
    function mint(
        address artist,
        string memory tokenURI,
        string memory isrc,
        string memory cid,
        uint8 band,
        string memory genre
    ) public onlyOwner returns (uint256) {
        uint256 tokenId = _nextTokenId++;
        
        _safeMint(artist, tokenId);
        _setTokenURI(tokenId, tokenURI);
        
        tokenMetadata[tokenId] = Metadata({
            isrc: isrc,
            cid: cid,
            band: band,
            releaseDate: block.timestamp,
            genre: genre
        });

        creatorOf[tokenId] = artist;

        // Set default royalties: 5% to the creator
        _setTokenRoyalty(tokenId, artist, 500);

        emit Minted(tokenId, artist, isrc, cid);
        return tokenId;
    }

    // Overrides required by Solidity for multiple inheritance

    function tokenURI(uint256 tokenId)
        public
        view
        override(ERC721, ERC721URIStorage)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(ERC721URIStorage, ERC2981)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
}
