<?xml version="1.0" encoding="UTF-8"?>
<!--
  work_registration.xsl — Identity transform / canonical passthrough.
  Input:  <WorkRegistrations> document (Retrosync canonical CWR-XML).
  Output: unchanged — used as base template for all society stylesheets.
  XSLT 1.0 (compatible with xot / libxslt).
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1">
  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:template match="@*|node()">
    <xsl:copy>
      <xsl:apply-templates select="@*|node()"/>
    </xsl:copy>
  </xsl:template>
</xsl:stylesheet>
