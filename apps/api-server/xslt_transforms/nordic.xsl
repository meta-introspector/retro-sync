<?xml version="1.0" encoding="UTF-8"?>
<!--
  nordic.xsl — Transforms Retrosync canonical CWR-XML into the NCB
  Nordic works registration XML shared by STIM (SE), TONO (NO), KODA (DK),
  TEOSTO (FI), STEF (IS).  CWR codes: 077, 083, 040, 078, 113.
  Format: ncb.dk/nordic/works/1.0
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:ncb="https://www.ncb.dk/nordic/works/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="sender_id"   select="'RETROSYNC'"/>
  <xsl:param name="created_date" select="'19700101'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <ncb:NordicWorkSubmission>
      <ncb:Header>
        <ncb:SenderID><xsl:value-of select="$sender_id"/></ncb:SenderID>
        <ncb:CreatedDate><xsl:value-of select="$created_date"/></ncb:CreatedDate>
        <ncb:Societies>
          <ncb:Society code="077">STIM</ncb:Society>
          <ncb:Society code="083">TONO</ncb:Society>
          <ncb:Society code="040">KODA</ncb:Society>
          <ncb:Society code="078">TEOSTO</ncb:Society>
          <ncb:Society code="113">STEF</ncb:Society>
        </ncb:Societies>
        <ncb:WorkCount><xsl:value-of select="count(cwr:Work)"/></ncb:WorkCount>
      </ncb:Header>
      <ncb:Works>
        <xsl:apply-templates select="cwr:Work"/>
      </ncb:Works>
    </ncb:NordicWorkSubmission>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <ncb:Work>
      <ncb:ISWC><xsl:value-of select="cwr:Iswc"/></ncb:ISWC>
      <ncb:Title><xsl:value-of select="cwr:Title"/></ncb:Title>
      <ncb:Language><xsl:value-of select="cwr:Language"/></ncb:Language>
      <ncb:Arrangement><xsl:value-of select="cwr:MusicArrangement"/></ncb:Arrangement>
      <ncb:Writers>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </ncb:Writers>
      <ncb:Publishers>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </ncb:Publishers>
      <xsl:if test="cwr:Recording">
        <ncb:ISRC><xsl:value-of select="cwr:Recording/cwr:Isrc"/></ncb:ISRC>
      </xsl:if>
    </ncb:Work>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <ncb:Writer role="{cwr:Role}" share="{cwr:SharePct}" society="{cwr:Society}">
      <ncb:Name><xsl:value-of select="concat(cwr:FirstName, ' ', cwr:LastName)"/></ncb:Name>
      <ncb:IPI><xsl:value-of select="cwr:IpiCae"/></ncb:IPI>
    </ncb:Writer>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <ncb:Publisher share="{cwr:SharePct}">
      <ncb:Name><xsl:value-of select="cwr:Name"/></ncb:Name>
      <ncb:IPI><xsl:value-of select="cwr:IpiCae"/></ncb:IPI>
    </ncb:Publisher>
  </xsl:template>
</xsl:stylesheet>
