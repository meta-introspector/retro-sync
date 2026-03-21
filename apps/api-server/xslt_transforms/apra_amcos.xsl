<?xml version="1.0" encoding="UTF-8"?>
<!--
  apra_amcos.xsl — Transforms Retrosync canonical CWR-XML into the
  APRA AMCOS electronic submission envelope (apra.com.au/cwr/submission/1.0).
  Societies: APRA AMCOS (AU/NZ).  CWR society code: 006.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:apra="https://www.apra.com.au/cwr/submission/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="sender_id" select="'RETROSYNC'"/>
  <xsl:param name="submission_date" select="'19700101'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <apra:APRASubmission>
      <apra:Header>
        <apra:SenderID><xsl:value-of select="$sender_id"/></apra:SenderID>
        <apra:SubmissionDate><xsl:value-of select="$submission_date"/></apra:SubmissionDate>
        <apra:Format>CWR</apra:Format>
        <apra:Version>2.2</apra:Version>
        <apra:WorkCount><xsl:value-of select="count(cwr:Work)"/></apra:WorkCount>
      </apra:Header>
      <apra:Works>
        <xsl:apply-templates select="cwr:Work"/>
      </apra:Works>
    </apra:APRASubmission>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <apra:Work>
      <apra:ISWC><xsl:value-of select="cwr:Iswc"/></apra:ISWC>
      <apra:Title><xsl:value-of select="cwr:Title"/></apra:Title>
      <apra:Language><xsl:value-of select="cwr:Language"/></apra:Language>
      <apra:MusicArrangement><xsl:value-of select="cwr:MusicArrangement"/></apra:MusicArrangement>
      <apra:Writers>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </apra:Writers>
      <apra:Publishers>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </apra:Publishers>
      <xsl:if test="cwr:AlternateTitles/cwr:AlternateTitle">
        <apra:AlternateTitles>
          <xsl:apply-templates select="cwr:AlternateTitles/cwr:AlternateTitle"/>
        </apra:AlternateTitles>
      </xsl:if>
      <xsl:if test="cwr:Recording">
        <xsl:apply-templates select="cwr:Recording"/>
      </xsl:if>
    </apra:Work>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <apra:Writer>
      <apra:LastName><xsl:value-of select="cwr:LastName"/></apra:LastName>
      <apra:FirstName><xsl:value-of select="cwr:FirstName"/></apra:FirstName>
      <apra:IPI><xsl:value-of select="cwr:IpiCae"/></apra:IPI>
      <apra:Role><xsl:value-of select="cwr:Role"/></apra:Role>
      <apra:SharePct><xsl:value-of select="cwr:SharePct"/></apra:SharePct>
      <apra:Society><xsl:value-of select="cwr:Society"/></apra:Society>
    </apra:Writer>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <apra:Publisher>
      <apra:Name><xsl:value-of select="cwr:Name"/></apra:Name>
      <apra:IPI><xsl:value-of select="cwr:IpiCae"/></apra:IPI>
      <apra:SharePct><xsl:value-of select="cwr:SharePct"/></apra:SharePct>
      <apra:Society><xsl:value-of select="cwr:Society"/></apra:Society>
    </apra:Publisher>
  </xsl:template>

  <xsl:template match="cwr:AlternateTitle">
    <apra:AlternateTitle type="{cwr:TitleType}">
      <xsl:value-of select="cwr:Title"/>
    </apra:AlternateTitle>
  </xsl:template>

  <xsl:template match="cwr:Recording">
    <apra:Recording>
      <apra:ISRC><xsl:value-of select="cwr:Isrc"/></apra:ISRC>
      <apra:Label><xsl:value-of select="cwr:Label"/></apra:Label>
      <apra:ReleaseDate><xsl:value-of select="cwr:ReleaseDate"/></apra:ReleaseDate>
    </apra:Recording>
  </xsl:template>
</xsl:stylesheet>
