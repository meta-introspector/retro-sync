<?xml version="1.0" encoding="UTF-8"?>
<!--
  prs.xsl — Transforms Retrosync canonical CWR-XML into PRS Online
  work registration XML (prsformusic.com/works/registration/1.0).
  Societies: PRS for Music + MCPS (UK).  CWR codes: 052, 053.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:prs="https://www.prsformusic.com/works/registration/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="sender_id"   select="'RETROSYNC'"/>
  <xsl:param name="created_date" select="'19700101'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <prs:WorkRegistrationBatch>
      <prs:Header>
        <prs:SenderID><xsl:value-of select="$sender_id"/></prs:SenderID>
        <prs:CreatedDate><xsl:value-of select="$created_date"/></prs:CreatedDate>
        <prs:WorkCount><xsl:value-of select="count(cwr:Work)"/></prs:WorkCount>
      </prs:Header>
      <prs:Works>
        <xsl:apply-templates select="cwr:Work"/>
      </prs:Works>
    </prs:WorkRegistrationBatch>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <prs:Work>
      <prs:ISWC><xsl:value-of select="cwr:Iswc"/></prs:ISWC>
      <prs:Title><xsl:value-of select="cwr:Title"/></prs:Title>
      <prs:Language><xsl:value-of select="cwr:Language"/></prs:Language>
      <prs:MusicArrangement><xsl:value-of select="cwr:MusicArrangement"/></prs:MusicArrangement>
      <prs:GrandRights><xsl:value-of select="cwr:GrandRightsInd"/></prs:GrandRights>
      <xsl:if test="cwr:OpusNumber != ''">
        <prs:OpusNumber><xsl:value-of select="cwr:OpusNumber"/></prs:OpusNumber>
      </xsl:if>
      <prs:Writers>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </prs:Writers>
      <prs:Publishers>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </prs:Publishers>
      <xsl:if test="cwr:AlternateTitles/cwr:AlternateTitle">
        <prs:AlternateTitles>
          <xsl:apply-templates select="cwr:AlternateTitles/cwr:AlternateTitle"/>
        </prs:AlternateTitles>
      </xsl:if>
      <xsl:if test="cwr:Recording">
        <prs:Recording>
          <prs:ISRC><xsl:value-of select="cwr:Recording/cwr:Isrc"/></prs:ISRC>
          <prs:Label><xsl:value-of select="cwr:Recording/cwr:Label"/></prs:Label>
          <prs:ReleaseDate><xsl:value-of select="cwr:Recording/cwr:ReleaseDate"/></prs:ReleaseDate>
        </prs:Recording>
      </xsl:if>
    </prs:Work>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <prs:Writer>
      <prs:LastName><xsl:value-of select="cwr:LastName"/></prs:LastName>
      <prs:FirstName><xsl:value-of select="cwr:FirstName"/></prs:FirstName>
      <prs:IPI><xsl:value-of select="cwr:IpiCae"/></prs:IPI>
      <prs:IPIBase><xsl:value-of select="cwr:IpiBase"/></prs:IPIBase>
      <prs:Role><xsl:value-of select="cwr:Role"/></prs:Role>
      <prs:SharePct><xsl:value-of select="cwr:SharePct"/></prs:SharePct>
      <prs:Society><xsl:value-of select="cwr:Society"/></prs:Society>
      <prs:Controlled><xsl:value-of select="cwr:Controlled"/></prs:Controlled>
    </prs:Writer>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <prs:Publisher>
      <prs:Name><xsl:value-of select="cwr:Name"/></prs:Name>
      <prs:IPI><xsl:value-of select="cwr:IpiCae"/></prs:IPI>
      <prs:IPIBase><xsl:value-of select="cwr:IpiBase"/></prs:IPIBase>
      <prs:Type><xsl:value-of select="cwr:PublisherType"/></prs:Type>
      <prs:SharePct><xsl:value-of select="cwr:SharePct"/></prs:SharePct>
    </prs:Publisher>
  </xsl:template>

  <xsl:template match="cwr:AlternateTitle">
    <prs:AlternateTitle type="{cwr:TitleType}" lang="{cwr:Language}">
      <xsl:value-of select="cwr:Title"/>
    </prs:AlternateTitle>
  </xsl:template>
</xsl:stylesheet>
