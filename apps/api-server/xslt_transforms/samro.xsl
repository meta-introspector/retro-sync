<?xml version="1.0" encoding="UTF-8"?>
<!--
  samro.xsl — Transforms Retrosync canonical CWR-XML into SAMRO
  online work registration XML (samro.org.za/works/1.0).
  Society: SAMRO (ZA).  CWR code: 066.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:samro="https://www.samro.org.za/works/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="submitter_id" select="'RETROSYNC'"/>
  <xsl:param name="submission_date" select="'19700101'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <samro:WorkSubmission>
      <samro:Header>
        <samro:SubmitterID><xsl:value-of select="$submitter_id"/></samro:SubmitterID>
        <samro:SubmissionDate><xsl:value-of select="$submission_date"/></samro:SubmissionDate>
        <samro:WorkCount><xsl:value-of select="count(cwr:Work)"/></samro:WorkCount>
        <samro:Territory>ZA</samro:Territory>
      </samro:Header>
      <samro:Works>
        <xsl:apply-templates select="cwr:Work"/>
      </samro:Works>
    </samro:WorkSubmission>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <samro:Work>
      <samro:ISWC><xsl:value-of select="cwr:Iswc"/></samro:ISWC>
      <samro:Title><xsl:value-of select="cwr:Title"/></samro:Title>
      <samro:Language><xsl:value-of select="cwr:Language"/></samro:Language>
      <samro:Arrangement><xsl:value-of select="cwr:MusicArrangement"/></samro:Arrangement>
      <samro:Authors>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </samro:Authors>
      <samro:Publishers>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </samro:Publishers>
      <xsl:if test="cwr:Recording/cwr:Isrc">
        <samro:ISRC><xsl:value-of select="cwr:Recording/cwr:Isrc"/></samro:ISRC>
      </xsl:if>
    </samro:Work>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <samro:Author role="{cwr:Role}" share="{cwr:SharePct}">
      <samro:LastName><xsl:value-of select="cwr:LastName"/></samro:LastName>
      <samro:FirstName><xsl:value-of select="cwr:FirstName"/></samro:FirstName>
      <samro:IPI><xsl:value-of select="cwr:IpiCae"/></samro:IPI>
    </samro:Author>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <samro:Publisher share="{cwr:SharePct}">
      <samro:Name><xsl:value-of select="cwr:Name"/></samro:Name>
      <samro:IPI><xsl:value-of select="cwr:IpiCae"/></samro:IPI>
    </samro:Publisher>
  </xsl:template>
</xsl:stylesheet>
