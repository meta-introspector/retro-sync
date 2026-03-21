<?xml version="1.0" encoding="UTF-8"?>
<!--
  jasrac.xsl — Transforms Retrosync canonical CWR-XML into JASRAC
  J-DISC XML upload format (jasrac.or.jp/jdisc/1.0).
  Society: JASRAC (JP).  CWR code: 099.
  Note: J-number assigned by JASRAC post-registration; left empty on first submission.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:jd="https://www.jasrac.or.jp/jdisc/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="publisher_ipi" select="''"/>
  <xsl:param name="registration_date" select="'19700101'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <jd:JDiscRegistration>
      <jd:Header>
        <jd:PublisherIPI><xsl:value-of select="$publisher_ipi"/></jd:PublisherIPI>
        <jd:RegistrationDate><xsl:value-of select="$registration_date"/></jd:RegistrationDate>
        <jd:WorkCount><xsl:value-of select="count(cwr:Work)"/></jd:WorkCount>
      </jd:Header>
      <jd:Works>
        <xsl:apply-templates select="cwr:Work"/>
      </jd:Works>
    </jd:JDiscRegistration>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <jd:Sakuhin>
      <jd:JasracCode/>
      <jd:ISWC><xsl:value-of select="cwr:Iswc"/></jd:ISWC>
      <jd:Title><xsl:value-of select="cwr:Title"/></jd:Title>
      <xsl:if test="cwr:AlternateTitles/cwr:AlternateTitle[cwr:TitleType='OT']">
        <jd:OriginalTitle>
          <xsl:value-of select="cwr:AlternateTitles/cwr:AlternateTitle[cwr:TitleType='OT']/cwr:Title"/>
        </jd:OriginalTitle>
      </xsl:if>
      <jd:Language><xsl:value-of select="cwr:Language"/></jd:Language>
      <jd:Arrangement><xsl:value-of select="cwr:MusicArrangement"/></jd:Arrangement>
      <jd:Sakkyokuka>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer[cwr:Role='C' or cwr:Role='CA']"/>
      </jd:Sakkyokuka>
      <jd:Sakushika>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer[cwr:Role='A' or cwr:Role='CA']"/>
      </jd:Sakushika>
      <jd:Shuppansha>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </jd:Shuppansha>
      <xsl:if test="cwr:PerformingArtists/cwr:PerformingArtist">
        <jd:Jisshika>
          <xsl:apply-templates select="cwr:PerformingArtists/cwr:PerformingArtist"/>
        </jd:Jisshika>
      </xsl:if>
      <xsl:if test="cwr:Recording/cwr:Isrc">
        <jd:ISRC><xsl:value-of select="cwr:Recording/cwr:Isrc"/></jd:ISRC>
      </xsl:if>
    </jd:Sakuhin>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <jd:Person>
      <jd:LastName><xsl:value-of select="cwr:LastName"/></jd:LastName>
      <jd:FirstName><xsl:value-of select="cwr:FirstName"/></jd:FirstName>
      <jd:IPI><xsl:value-of select="cwr:IpiCae"/></jd:IPI>
      <jd:Role><xsl:value-of select="cwr:Role"/></jd:Role>
      <jd:Share><xsl:value-of select="cwr:SharePct"/></jd:Share>
    </jd:Person>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <jd:Publisher>
      <jd:Name><xsl:value-of select="cwr:Name"/></jd:Name>
      <jd:IPI><xsl:value-of select="cwr:IpiCae"/></jd:IPI>
      <jd:Share><xsl:value-of select="cwr:SharePct"/></jd:Share>
    </jd:Publisher>
  </xsl:template>

  <xsl:template match="cwr:PerformingArtist">
    <jd:Artist>
      <jd:Name><xsl:value-of select="cwr:LastName"/></jd:Name>
      <xsl:if test="cwr:Isni">
        <jd:ISNI><xsl:value-of select="cwr:Isni"/></jd:ISNI>
      </xsl:if>
    </jd:Artist>
  </xsl:template>
</xsl:stylesheet>
