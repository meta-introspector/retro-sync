<?xml version="1.0" encoding="UTF-8"?>
<!--
  socan.xsl — Transforms Retrosync canonical CWR-XML into SOCAN
  MusicMark submission XML (socan.ca/musicmark/1.0).
  Societies: SOCAN + CMRRA (CA).  CWR codes: 055, 050.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:socan="https://www.socan.ca/musicmark/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="sender_id"   select="'RETROSYNC'"/>
  <xsl:param name="created"     select="'1970-01-01T00:00:00Z'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <socan:MusicMarkSubmission>
      <socan:Header>
        <socan:SenderID><xsl:value-of select="$sender_id"/></socan:SenderID>
        <socan:Created><xsl:value-of select="$created"/></socan:Created>
        <socan:WorkCount><xsl:value-of select="count(cwr:Work)"/></socan:WorkCount>
      </socan:Header>
      <socan:Works>
        <xsl:apply-templates select="cwr:Work"/>
      </socan:Works>
    </socan:MusicMarkSubmission>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <socan:Work>
      <socan:ISWC><xsl:value-of select="cwr:Iswc"/></socan:ISWC>
      <socan:Title><xsl:value-of select="cwr:Title"/></socan:Title>
      <socan:Language><xsl:value-of select="cwr:Language"/></socan:Language>
      <socan:Writers>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </socan:Writers>
      <socan:Publishers>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </socan:Publishers>
      <socan:Territories>
        <xsl:apply-templates select="cwr:Territories/cwr:Territory"/>
      </socan:Territories>
    </socan:Work>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <socan:Writer ipi="{cwr:IpiCae}" role="{cwr:Role}" share="{cwr:SharePct}"
                  society="{cwr:Society}">
      <xsl:value-of select="concat(cwr:FirstName, ' ', cwr:LastName)"/>
    </socan:Writer>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <socan:Publisher ipi="{cwr:IpiCae}" share="{cwr:SharePct}">
      <xsl:value-of select="cwr:Name"/>
    </socan:Publisher>
  </xsl:template>

  <xsl:template match="cwr:Territory">
    <socan:Territory tis="{cwr:TisCode}"/>
  </xsl:template>
</xsl:stylesheet>
